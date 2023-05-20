use crate::PdfExtractor;
use anyhow::{anyhow, Context};
use async_trait::async_trait;

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use std::sync::Arc;
use tracing::log::error;

use url::Url;

use shared_kernel::http_client::HttpClient;

use use_cases::import_affected_areas::Region;

lazy_static! {
    static ref FORWARD_SLASH: Regex =
        Regex::new("/").expect("Expected regex to compile forward slash pattern");
}

#[async_trait]
pub trait TextExtractor: Send + Sync {
    async fn extract(&self, text: String) -> anyhow::Result<Vec<Region>>;
}

pub struct PdfExtractorImpl {
    text_extractor: Arc<dyn TextExtractor>,
}

impl PdfExtractorImpl {
    pub fn new(text_extractor: Arc<dyn TextExtractor>) -> Self {
        Self { text_extractor }
    }
    async fn fetch_and_extract(&self, url: Url) -> anyhow::Result<(Url, Vec<Region>)> {
        let res = HttpClient::get_bytes(url.clone()).await?;
        let text = resolve_text_from_file(&res)
            .await
            .with_context(|| format!("The file URL is {url}"))?;
        let regions = self.text_extractor.extract(text).await?;

        Ok((url, regions))
    }
}

#[async_trait]
impl PdfExtractor for PdfExtractorImpl {
    #[tracing::instrument(err, skip(self), level = "info")]
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
            error!("{errors:?}")
        }
        if results.is_empty() && !errors.is_empty() {
            return Err(anyhow!("{errors:?}"));
        }

        Ok(results)
    }
}

async fn resolve_text_from_file(file_bytes: &[u8]) -> anyhow::Result<String> {
    use pdf_extract::*;
    let result = extract_text_from_mem(file_bytes).context("Failed to extract pdf to text")?;
    Ok(result)
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
                Url::parse("https://kplc.co.ke/img/full/Interruption%20-%2011.05.2023.pdf")
                    .unwrap(),
            ];
        let _res = extractor.extract(links).await.expect("Expected result");
    }
}
