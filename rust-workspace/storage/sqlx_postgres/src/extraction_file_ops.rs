use std::collections::HashSet;

use crate::repository::Repository;
use async_trait::async_trait;
use itertools::Itertools;
use url::Url;
use web_page_extractor::FileOperations;

use anyhow::Context;

#[async_trait]
impl FileOperations for Repository {
    async fn return_unprocessed_files(&self, files: Vec<Url>) -> anyhow::Result<Vec<Url>> {
        let keys = files.into_iter().map(|file| file.to_string()).collect_vec();

        let records = sqlx::query!(
            "
            SELECT url FROM source WHERE url = ANY($1)
            ",
            &keys[..]
        )
        .fetch_all(self.pool())
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
}
