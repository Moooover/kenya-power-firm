use crate::repository::Repository;
use anyhow::Context;
use entities::power_interruptions::location::{AffectedLine, Region};
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::Row;

struct SearcheableLine(String);

impl AsRef<str> for SearcheableLine {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&String> for SearcheableLine {
    fn from(value: &String) -> Self {
        let value = value.trim().replace(' ', " & ");
        SearcheableLine(value)
    }
}

#[derive(sqlx::FromRow, Debug)]
struct DbLocationSearchResults {
    search_query: String,
    location: String,
    id: uuid::Uuid,
}

impl Repository {
    async fn search_for_affected_lines(
        &self,
        regions: &[Region],
    ) -> anyhow::Result<Vec<AffectedLine>> {
        let counties = regions.iter().flat_map(|region| &region.counties);
        let areas = counties.flat_map(|county| &county.areas);
        let lines = areas.flat_map(|area| {
            // TODO: Rename to lines
            &area.locations
        });
        let searcheable_lines = lines.map_into().collect::<Vec<SearcheableLine>>();
        let searcheable_lines = searcheable_lines
            .iter()
            .map(|line| line.as_ref())
            .collect::<Vec<_>>();
        let pool = self.pool();

        let mut rows = sqlx::query_as::<_, DbLocationSearchResults>(
            "
            SELECT * FROM location.search_locations_primary_text($1::text[])
            ",
        )
        .bind(searcheable_lines)
        .fetch(pool);

        while let Some(line) = rows.try_next().await? {
            println!("{line:?}")
        }

        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use entities::{
        locations::LocationInput,
        power_interruptions::location::{Area, County, NairobiTZDateTime, Region, TimeFrame},
    };

    use crate::repository::Repository;

    fn generate_region() -> Region {
        Region {
            region: "Nairobi".to_string(),
            counties: vec![County {
                name: "Nairobi".to_string(),
                areas: vec![
                    Area {
                        name: "Garden city".to_string(),
                        time_frame: TimeFrame {
                            from: NairobiTZDateTime::today().try_into().unwrap(),
                            to: NairobiTZDateTime::today().try_into().unwrap(),
                        },
                        locations: vec!["Will Mary Estate".to_string(), "Mi Vida".to_string()],
                    },
                    Area {
                        name: "Lumumba".to_string(),
                        time_frame: TimeFrame {
                            from: NairobiTZDateTime::today().try_into().unwrap(),
                            to: NairobiTZDateTime::today().try_into().unwrap(),
                        },
                        locations: vec![
                            "Lumumba dr".to_string(),
                            "Pan Africa Christian University".to_string(),
                        ],
                    },
                ],
            }],
        }
    }

    #[tokio::test]
    async fn test_searching_for_locations_works() {
        let repository = Repository::new_test_repo().await;
        // INFO: From this test, it appears to me that the location inserted by the user needs
        // a lot of data in order to correctly match it.
        repository
            .insert_location(LocationInput {
                name: "Mi Vida, Garden City".to_string(),
                address: "".to_string(),
                external_id: "".to_string().into(),
                api_response: serde_json::json!({}),
            })
            .await
            .unwrap();
        repository
            .insert_location(LocationInput {
                name: "Beston Fast foods, Lumumba Drive".to_string(),
                address: "".to_string(),
                external_id: "".to_string().into(),
                api_response: serde_json::json!({}),
            })
            .await
            .unwrap();
        let region = generate_region();
        let results = repository.search_for_affected_lines(&[region]).await;

        assert!(results.is_ok())
    }
}
