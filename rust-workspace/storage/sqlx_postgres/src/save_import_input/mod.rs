mod counties;
use anyhow::Context;

use sqlx::query;
use std::{
    collections::{HashMap, HashSet},
    iter,
};

use crate::repository::Repository;
use crate::save_import_input::counties::{DbCounty, DbCountyId};
use async_trait::async_trait;
use entities::power_interruptions::location::{
    Area, AreaName, FutureOrCurrentNairobiTZDateTime, NairobiTZDateTime, Region,
};
use url::Url;
use use_cases::import_affected_areas::SaveBlackoutAffectedAreasRepo;
use uuid::Uuid;

#[async_trait]
impl SaveBlackoutAffectedAreasRepo for Repository {
    async fn save(
        &self,
        data: &entities::power_interruptions::location::ImportInput,
    ) -> anyhow::Result<()> {
        let counties = self.get_counties().await?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("Failed to begin transaction")?;

        for (url, regions) in data.iter() {
            let source = SourceFile::save(&mut transaction, url).await?;

            save_regions_data(regions, &counties, &mut transaction, source.id).await?;
        }

        transaction
            .commit()
            .await
            .context("Failed to commit transaction")?;

        Ok(())
    }
}

async fn save_regions_data(
    regions: &[Region],
    counties: &[DbCounty],
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    source_id: Uuid,
) -> anyhow::Result<()> {
    let counties = regions
        .iter()
        .flat_map(|region| region.counties.clone())
        .map(|county| DbCounty::resolve_county_id(counties, &county.name).map(|id| (id, county)))
        .collect::<Result<Vec<_>, _>>()?;

    let county_ids_with_areas = counties
        .into_iter()
        .flat_map(|(county_id, county)| county.areas.into_iter().map(move |area| (county_id, area)))
        .collect::<Vec<_>>();

    let areas = AreaWithId::save_many(transaction, county_ids_with_areas)
        .await
        .context("Failed to save & return areas")?;

    let area_id_to_blackout_schedule =
        BlackoutSchedule::save_many(transaction, source_id, &areas).await?;

    let lines = areas
        .iter()
        .filter_map(|data| {
            area_id_to_blackout_schedule
                .get(&AreaId(data.id))
                .map(|blackout_schedule| {
                    // TODO: Streamline the language here, we can go with line instead of location;
                    data.area.locations.iter().map(|line| DbLine {
                        area_id: data.id,
                        name: line.clone(),
                        blackout_schedule: blackout_schedule.0,
                    })
                })
        })
        .flatten()
        .collect::<HashSet<_>>();

    DbLine::save_many(transaction, lines)
        .await
        .context("Failed to save lines")?;

    Ok(())
}

struct SourceFile {
    id: Uuid,
}

impl SourceFile {
    async fn save(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        url: &Url,
    ) -> anyhow::Result<Self> {
        let url = url.as_str();
        let _ = sqlx::query!(
            "INSERT INTO public.source(url) VALUES ($1) ON CONFLICT DO NOTHING",
            url
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to insert source")?;

        let source = sqlx::query!("SELECT id, url FROM public.source WHERE url = $1", url)
            .fetch_one(&mut *transaction)
            .await
            .context("Failed to fetch source")?;

        Ok(SourceFile { id: source.id })
    }
}

struct AreaWithId {
    id: Uuid,
    area: Area<FutureOrCurrentNairobiTZDateTime>,
}

impl AreaWithId {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        county_id_with_areas: Vec<(DbCountyId, Area<FutureOrCurrentNairobiTZDateTime>)>,
    ) -> Result<Vec<AreaWithId>, sqlx::Error> {
        let (area_names, county_ids): (Vec<_>, Vec<_>) = county_id_with_areas
            .iter()
            .map(|(id, area)| (area.name.to_string(), id.inner()))
            .unzip();

        sqlx::query!(
            "
            INSERT INTO location.area(name, county_id)
            SELECT * FROM UNNEST($1::text[], $2::uuid[]) ON CONFLICT DO NOTHING
            ",
            &area_names[..],
            &county_ids[..]
        )
        .execute(&mut *transaction)
        .await?;
        let inserted_areas = query!(
            "SELECT * FROM location.area WHERE name = ANY($1)",
            &area_names[..]
        )
        .fetch_all(&mut *transaction)
        .await?;
        let mapping_of_area_name_to_id = inserted_areas
            .into_iter()
            .map(|record| (AreaName::from(record.name), record.id))
            .collect::<HashMap<_, _>>();
        Ok(county_id_with_areas
            .into_iter()
            .filter_map(|(_id, area)| {
                mapping_of_area_name_to_id
                    .get(&area.name)
                    .map(|id| AreaWithId { id: *id, area })
            })
            .collect::<Vec<_>>())
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct DbLine {
    area_id: Uuid,
    name: String,
    blackout_schedule: Uuid,
}

impl DbLine {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        lines: HashSet<DbLine>,
    ) -> anyhow::Result<()> {
        let (line_names, line_area_ids): (Vec<_>, Vec<_>) = lines
            .iter()
            .map(|line| (line.name.clone(), line.area_id))
            .unzip();

        sqlx::query!(
            "
            INSERT INTO location.line(name, area_id)
            SELECT * FROM UNNEST($1::text[], $2::uuid[]) ON CONFLICT DO NOTHING
            ",
            &line_names[..],
            &line_area_ids[..]
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to insert lines")?;

        let inserted_lines = query!(
            "SELECT id, name FROM location.line WHERE name = ANY($1) AND area_id = ANY($2)",
            &line_names[..],
            &line_area_ids[..]
        )
        .fetch_all(&mut *transaction)
        .await
        .context("Failed to return inserted lines")?;

        let mapping_of_line_id_to_name = inserted_lines
            .into_iter()
            .map(|line| (line.name, line.id))
            .collect::<HashMap<_, _>>();

        let (line_ids, schedule_ids): (Vec<_>, Vec<_>) = lines
            .iter()
            .filter_map(|line| {
                mapping_of_line_id_to_name
                    .get(&line.name)
                    .map(|line_id| (*line_id, line.blackout_schedule))
            })
            .unzip();

        sqlx::query!(
            "
            INSERT INTO location.line_schedule(line_id, schedule_id)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[]) ON CONFLICT DO NOTHING
            ",
            &line_ids[..],
            &schedule_ids[..]
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to insert line schedules")?;

        Ok(())
    }
}

pub struct BlackoutSchedule {
    pub id: Uuid,
    pub area_id: Uuid,
    pub start_time: NairobiTZDateTime,
    pub end_time: NairobiTZDateTime,
}

#[derive(PartialEq, Eq, Hash)]
pub struct AreaId(Uuid);

#[derive(PartialEq, Eq, Hash)]
pub struct BlackoutScheduleId(Uuid);

impl BlackoutSchedule {
    async fn save_many(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        source_id: Uuid,
        areas: &[AreaWithId],
    ) -> anyhow::Result<HashMap<AreaId, BlackoutScheduleId>> {
        let area_ids = areas.iter().map(|area| area.id).collect::<Vec<_>>();
        let (start_times, end_times): (Vec<_>, Vec<_>) = areas
            .iter()
            .map(|area| {
                let time_frame = &area.area.time_frame;
                (time_frame.from.to_date_time(), time_frame.to.to_date_time())
            })
            .unzip();
        let source_ids = iter::repeat(source_id)
            .take(areas.len())
            .collect::<Vec<_>>();

        sqlx::query!(
            "
            INSERT INTO location.blackout_schedule(area_id, start_time, end_time, source_id) 
            SELECT * FROM UNNEST($1::uuid[], $2::timestamptz[], $3::timestamptz[], $4::uuid[]) ON CONFLICT DO NOTHING
            ",
            &area_ids[..],
            &start_times[..],
            &end_times[..],
            &source_ids[..]
        )
        .execute(&mut *transaction).await.context("Failed to save schedules")?;

        let inserted_area_schedules = query!(
            "SELECT id, area_id FROM location.blackout_schedule WHERE area_id = ANY($1) AND source_id = $2",
            &area_ids[..],
            source_id
        )
        .fetch_all(&mut *transaction)
        .await?;

        let results = inserted_area_schedules
            .into_iter()
            .map(|data| (AreaId(data.area_id), BlackoutScheduleId(data.id)))
            .collect::<HashMap<_, _>>();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use entities::power_interruptions::location::{
        Area, County, ImportInput, NairobiTZDateTime, Region, TimeFrame,
    };
    use use_cases::import_affected_areas::SaveBlackoutAffectedAreasRepo;

    use crate::repository::Repository;

    fn generate_input() -> ImportInput {
        let url = url::Url::parse("https://crates.io/crates/fake").unwrap();
        let region = Region {
            region: "Nairobi".to_string(),
            counties: vec![County {
                name: "Nairobi".to_string(),
                areas: vec![Area {
                    name: "Garden city".to_string().into(),
                    time_frame: TimeFrame {
                        from: NairobiTZDateTime::today().try_into().unwrap(),
                        to: NairobiTZDateTime::today().try_into().unwrap(),
                    },
                    locations: vec!["Will Mary Estate".to_string(), "Mi Vida".to_string()],
                }],
            }],
        };
        let data = HashMap::from_iter([(url, vec![region])]);
        ImportInput::new(data)
    }
    #[tokio::test]
    async fn test_can_save_data_successfully() {
        let repository = Repository::new_test_repo().await;
        let result = repository.save(&generate_input()).await;
        assert!(result.is_ok())
    }
}
