mod content_extractor;

use anyhow::bail;
use entities::power_interruptions::location::Region;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::collections::HashMap;
use tracing::error;
use url::Url;

pub struct PdfReader;

impl PdfReader {
    pub fn new() -> Self {
        Self
    }

    #[tracing::instrument(err, skip(self), level = "info")]
    pub async fn extract(&self, links: Vec<Url>) -> anyhow::Result<HashMap<Url, Vec<Region>>> {
        let number_of_links = links.len();

        let mut futures: FuturesUnordered<_> = links
            .into_iter()
            .map(|url| fetch_and_extract::execute(url))
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
            bail!("{errors:?}")
        }

        Ok(results)
    }
}

mod fetch_and_extract {
    use crate::pdf_reader::content_extractor::extract;
    use anyhow::Context;
    use entities::power_interruptions::location::Region;
    use shared_kernel::http_client::HttpClient;
    use url::Url;

    pub(super) async fn execute(url: Url) -> anyhow::Result<(Url, Vec<Region>)> {
        use pdf_extract::*;
        let file_bytes = HttpClient::get_bytes(url.clone()).await?;
        let text = extract_text_from_mem(&file_bytes).context("Failed to extract pdf to text")?;
        let regions = extract(text)?;
        return Ok((url, regions));
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    #[tokio::test]
    async fn test_pdf_reader() {
        let url = Url::parse(
            "https://www.kplc.co.ke/img/full/Interruption%20Notices%20-%2018.05.2022.pdf",
        )
        .unwrap();
        let urls = vec![url];
        let pdf_reader = super::PdfReader::new();
        let result = pdf_reader.extract(urls).await.unwrap();
        println!("{:?}", result);
    }
}
