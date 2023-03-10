use crate::PdfExtractor;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use url::Url;

use shared_kernel::http_client::HttpClient;

use use_cases::import_affected_areas::{Area, Region};

lazy_static! {
    static ref FORWARD_SLASH: Regex =
        Regex::new("/").expect("Expected regex to compile forward slash pattern");
}

#[async_trait]
pub trait TextExtractor: Send + Sync {
    async fn extract(&self, text: String) -> anyhow::Result<Vec<Region>>;
}

struct PdfExtractorImpl {
    text_extractor: Arc<dyn TextExtractor>,
}

impl PdfExtractorImpl {
    async fn fetch_and_extract(&self, url: Url) -> anyhow::Result<(Url, Vec<Region>)> {
        let res = HttpClient::get_bytes(url.clone()).await?;
        let text = resolve_text_from_file(&url, &res).await?;
        let regions = self.text_extractor.extract(text).await?;

        Ok((url, regions))
    }
}

#[async_trait]
impl PdfExtractor for PdfExtractorImpl {
    async fn extract(&self, links: Vec<Url>) -> anyhow::Result<HashMap<Url, Vec<Region>>> {
        let number_of_links = links.len();

        let mut futures: FuturesUnordered<_> = links
            .into_iter()
            .map(|url| self.fetch_and_extract(url))
            .collect();

        let mut errors = vec![];
        let mut results = HashMap::with_capacity(number_of_links);

        while let Some(result) = futures.next().await {
            match result {
                Ok((url, regions)) => {
                    results.insert(url, regions);
                }
                Err(error) => errors.push(error),
            }
        }

        if !errors.is_empty() {
            // TODO: Setup logging
            println!("{errors:?}")
        }
        if results.is_empty() && !errors.is_empty() {
            return Err(anyhow!("{errors:?}"));
        }

        Ok(results)
    }
}

async fn resolve_text_from_file(url: &Url, file_bytes: &[u8]) -> anyhow::Result<String> {
    use pdf_extract::extract_text;
    use std::env;
    use tokio::fs::remove_file;
    use tokio::fs::File;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;

    let path = env::current_dir().context("Cannot read current_dir")?;
    let normalized_url = FORWARD_SLASH.replace_all(url.as_str(), "_");
    let file_path = format!("{}/pdf_dump/pdf-{}", path.display(), normalized_url);
    println!("{file_path}");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_path)
        .await
        .context("Failed to create file")?;

    file.write_all(file_bytes)
        .await
        .context("Failed to write pdf content to file")?;

    let result = extract_text(&file_path).context("Failed to extract pdf to text")?;

    tokio::spawn(delete_file(file_path));
    Ok(result)
}

async fn delete_file(path: String) {
    use tokio::fs::remove_file;

    let file_deleting_result = remove_file(&path)
        .await
        .context(format!("Failed to delete file {path} after extraction"));
    if let Err(err) = file_deleting_result {
        // TODO: Replace with tracing macro calls
        println!("{err}")
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf_extractor::{PdfExtractorImpl, TextExtractor};
    use crate::PdfExtractor;
    use async_trait::async_trait;
    use std::sync::Arc;
    use url::Url;
    use use_cases::import_affected_areas::Region;

    struct TestExtractor;

    #[async_trait]
    impl TextExtractor for TestExtractor {
        async fn extract(&self, text: String) -> anyhow::Result<Vec<Region>> {
            println!("{text}");

            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_working() {
        let extractor = PdfExtractorImpl {
            text_extractor: Arc::new(TestExtractor),
        };
        let links =
            vec![
                Url::parse("https://www.kplc.co.ke/img/full/Interruptions%20-%2023.02.2023.pdf")
                    .unwrap(),
            ];
        let res = extractor.extract(links).await.expect("Expected result");
    }
}
