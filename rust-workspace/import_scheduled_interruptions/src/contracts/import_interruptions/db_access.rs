use crate::db_access::DbAccess;
use anyhow::Context;
use entities::power_interruptions::location::ImportInput;

pub struct ImportPowerInterruptionsDbAccess {
    pub(crate) db: DbAccess,
}

impl ImportPowerInterruptionsDbAccess {
    pub fn new() -> Self {
        let db = DbAccess;
        Self { db }
    }

    pub(crate) async fn import(&self, data: &ImportInput) -> anyhow::Result<()> {
        let counties = counties::get_counties(&self).await?;
        let mut transaction = self
            .db
            .pool()
            .await
            .as_ref()
            .begin()
            .await
            .context("Failed to begin transaction")?;

        for (url, regions) in data.iter() {
            save_data::execute(regions, &counties, &mut transaction, url).await?;
        }

        transaction
            .commit()
            .await
            .context("Failed to commit transaction")?;

        Ok(())
    }
}

mod save_data {
    use crate::contracts::import_interruptions::db_access::counties::{DbCounty, DbCountyId};
    use anyhow::Context;
    use entities::power_interruptions::location::{
        Area, AreaName, FutureOrCurrentNairobiTZDateTime, NairobiTZDateTime, Region,
    };
    use sqlx::query;
    use std::collections::{HashMap, HashSet};
    use std::iter;
    use url::Url;
    use uuid::Uuid;

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

    pub(crate) async fn execute(
        regions: &[Region],
        counties: &[DbCounty],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        url: &Url,
    ) -> anyhow::Result<()> {
        let source_id = SourceFile::save(transaction, url).await?.id;

        save_regions_data(regions, counties, transaction, source_id).await
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

    async fn save_regions_data(
        regions: &[Region],
        counties: &[DbCounty],
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        source_id: Uuid,
    ) -> anyhow::Result<()> {
        let counties = regions
            .iter()
            .flat_map(|region| region.counties.clone())
            .map(|county| {
                DbCounty::resolve_county_id(counties, &county.name).map(|id| (id, county))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let county_ids_with_areas = counties
            .into_iter()
            .flat_map(|(county_id, county)| {
                county.areas.into_iter().map(move |area| (county_id, area))
            })
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
}

mod counties {
    use crate::contracts::import_interruptions::db_access::ImportPowerInterruptionsDbAccess;
    use anyhow::{anyhow, Context};
    use shared_kernel::uuid_key;

    pub struct DbCountyName(String);

    impl From<String> for DbCountyName {
        fn from(value: String) -> Self {
            DbCountyName(value)
        }
    }

    impl DbCountyName {
        fn matches(&self, county: &str) -> bool {
            if self.0.eq_ignore_ascii_case(county) {
                return true;
            }

            if let Some(first_part_of_county_name) =
                self.first_substring_of_county_name_split_by_space()
            {
                return county
                    .to_ascii_uppercase()
                    .contains(&first_part_of_county_name.to_ascii_uppercase());
            }

            false
        }

        fn first_substring_of_county_name_split_by_space(&self) -> Option<&str> {
            self.0.split(' ').collect::<Vec<_>>().first().cloned()
        }
    }

    uuid_key!(DbCountyId);

    pub struct DbCounty {
        pub id: DbCountyId,
        pub name: DbCountyName,
    }

    impl DbCounty {
        fn matches(&self, county: &str) -> bool {
            self.name.matches(county)
        }
    }

    impl DbCounty {
        pub fn resolve_county_id(
            counties: &[DbCounty],
            county: &str,
        ) -> anyhow::Result<DbCountyId> {
            for db_county in counties {
                if db_county.matches(county) {
                    return Ok(db_county.id);
                }
            }

            Err(anyhow!("Failed to resolve county ID {county}"))
        }
    }

    pub(super) async fn get_counties(
        db_access: &ImportPowerInterruptionsDbAccess,
    ) -> anyhow::Result<Vec<DbCounty>> {
        let pool = db_access.db.pool().await;

        let counties = sqlx::query!(
            "
            SELECT id, name FROM location.county
            "
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to load counties")?;

        let counties = counties
            .into_iter()
            .map(|record| DbCounty {
                id: record.id.into(),
                name: record.name.into(),
            })
            .collect();
        Ok(counties)
    }

    #[cfg(test)]
    mod tests {
        use crate::contracts::import_interruptions::db_access::counties::DbCounty;
        use uuid::Uuid;

        #[test]
        fn test_can_resolve_county_id_from_exact_matching_name() {
            let id = Uuid::new_v4().into();
            let db_county = DbCounty {
                id,
                name: "MIGORI".to_string().into(),
            };

            assert!(DbCounty::resolve_county_id(&[db_county], "MIGORI").is_ok())
        }

        #[test]
        fn test_if_county_is_a_subsstring_of_the_db_county_name() {
            let id = Uuid::new_v4().into();
            let db_county = DbCounty {
                id,
                name: "TAITA TAVETA".to_string().into(),
            };

            assert!(DbCounty::resolve_county_id(&[db_county], "TAITA").is_ok())
        }

        #[test]
        fn test_if_county_matches_if_county_name_is_separated_by_dash() {
            let id = Uuid::new_v4().into();
            let db_county = DbCounty {
                id,
                name: "TAITA TAVETA".to_string().into(),
            };

            assert!(DbCounty::resolve_county_id(&[db_county], "TAITA-TAVETA").is_ok())
        }
    }
}
