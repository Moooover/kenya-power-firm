use crate::repository::Repository;
use anyhow::{anyhow, Context};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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
        self.0.split(" ").collect::<Vec<_>>().first().cloned()
    }
}

#[derive(Copy, Clone)]
pub struct DbCountyId(Uuid);

impl DbCountyId {
    pub fn into_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for DbCountyId {
    fn from(value: Uuid) -> Self {
        DbCountyId(value)
    }
}

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
    pub fn resolve_county_id(counties: &[DbCounty], county: &str) -> anyhow::Result<DbCountyId> {
        for db_county in counties {
            if db_county.matches(county) {
                return Ok(db_county.id);
            }
        }

        Err(anyhow!("Failed to resolve county ID {county}"))
    }
}

impl Repository {
    pub async fn get_counties(&self) -> anyhow::Result<Vec<DbCounty>> {
        let pool = self.pool();
        let counties = sqlx::query!(
            "
            SELECT id, name FROM location.county
            "
        )
        .fetch_all(pool)
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
}

#[cfg(test)]
mod tests {
    use crate::save_import_input::counties::{DbCounty, DbCountyName};
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
