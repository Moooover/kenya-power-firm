use std::collections::HashSet;

use crate::repository::Repository;
use async_trait::async_trait;
use itertools::Itertools;
use url::Url;
use web_page_extractor::FileOperations;

use anyhow::Context;

#[async_trait]
impl FileOperations for Repository {
    async fn save_files(&self, files: Vec<Url>) -> anyhow::Result<()> {
        let keys = files.into_iter().map(|file| file.to_string()).collect_vec();
        sqlx::query!(
            "
              INSERT INTO importer.processed_files(id) 
              SELECT * FROM UNNEST($1::text[]) ON CONFLICT DO NOTHING
            ",
            &keys[..],
        )
        .execute(self.pool())
        .await
        .context("Failed to save_files")?;

        Ok(())
    }

    async fn return_unprocessed_files(&self, files: Vec<Url>) -> anyhow::Result<Vec<Url>> {
        let keys = files.into_iter().map(|file| file.to_string()).collect_vec();

        let records = sqlx::query!(
            "
            SELECT id FROM importer.processed_files WHERE id = ANY($1)
            ",
            &keys[..]
        )
        .fetch_all(self.pool())
        .await
        .context("Failed to fetch processed files")?;
        let all_provided_keys = keys.into_iter().collect::<HashSet<_>>();
        let existing_records = records
            .into_iter()
            .map(|record| record.id)
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
