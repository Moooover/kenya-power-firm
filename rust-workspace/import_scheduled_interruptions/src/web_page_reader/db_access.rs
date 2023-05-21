use crate::db_access::DbAccess;
use anyhow::Context;
use itertools::Itertools;
use std::collections::HashSet;
use url::Url;

pub struct WebPageReaderDbAccess {
    db: DbAccess,
}

impl WebPageReaderDbAccess {
    pub(crate) fn new() -> Self {
        Self { db: DbAccess }
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub(crate) async fn return_unprocessed_files(
        &self,
        files: Vec<Url>,
    ) -> anyhow::Result<Vec<Url>> {
        let pool = self.db.pool().await;
        let keys = files.into_iter().map(|file| file.to_string()).collect_vec();

        let records = sqlx::query!(
            "
            SELECT url FROM source WHERE url = ANY($1)
            ",
            &keys[..]
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to fetch urls from source table")?;
        let all_provided_keys = keys.into_iter().collect::<HashSet<_>>();
        let existing_records = records
            .into_iter()
            .map(|record| record.url)
            .collect::<HashSet<_>>();

        let (difference, errors): (Vec<_>, Vec<_>) = all_provided_keys
            .difference(&existing_records)
            .map(|url| Url::parse(url))
            .partition(Result::is_ok);

        let difference: Vec<_> = difference.into_iter().map(Result::unwrap).collect();
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

        if !errors.is_empty() {
            println!("Failed to parse some files as urls - {errors:?}")
        }

        Ok(difference)
    }

    pub(crate) async fn get_manually_added_source_files(&self) -> anyhow::Result<Vec<Url>> {
        let pool = self.db.pool().await;
        let records = sqlx::query!(
            "
                SELECT source_url FROM manually_added_sources
            "
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to fetch manually added source files")?;

        let urls = records
            .into_iter()
            .map(|record| Url::parse(&record.source_url))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse urls")?;

        Ok(urls)
    }
}
