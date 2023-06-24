mod content_extractor;

use anyhow::bail;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use shared_kernel::area_name::AreaName;
use shared_kernel::date_time::nairobi_date_time::FutureOrCurrentNairobiTZDateTime;
use shared_kernel::date_time::time_frame::TimeFrame;
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

        let mut futures: FuturesUnordered<_> =
            links.into_iter().map(fetch_and_extract::execute).collect();

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
    use crate::pdf_reader::Region;
    use anyhow::Context;
    use shared_kernel::http_client::HttpClient;
    use url::Url;

    pub(super) async fn execute(url: Url) -> anyhow::Result<(Url, Vec<Region>)> {
        use pdf_extract::*;
        let file_bytes = HttpClient::get_bytes(url.clone()).await?;
        let text = extract_text_from_mem(&file_bytes).context("Failed to extract pdf to text")?;
        let regions = extract(text)?;
        Ok((url, regions))
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    #[tokio::test]
    async fn test_pdf_reader() {
        let url =
            Url::parse("https://kplc.co.ke/img/full/Interruption%20-%2015.06.2023.pdf").unwrap();
        let urls = vec![url];
        let pdf_reader = super::PdfReader::new();
        let result = pdf_reader.extract(urls).await.unwrap();

        println!("{:?}", result);
    }
}

#[derive(Debug, Clone)]
pub struct County<T> {
    pub name: String,
    pub areas: Vec<Area<T>>,
}

#[derive(Debug)]
pub struct Region<T = FutureOrCurrentNairobiTZDateTime> {
    pub region: String,
    pub counties: Vec<County<T>>,
}

#[derive(Debug, Clone)]
pub struct Area<T> {
    pub name: AreaName,
    pub time_frame: TimeFrame<T>,
    pub locations: Vec<String>,
}

#[derive(Debug)]
pub struct ImportInput(HashMap<Url, Vec<Region<FutureOrCurrentNairobiTZDateTime>>>);

impl ImportInput {
    pub fn new(data: HashMap<Url, Vec<Region<FutureOrCurrentNairobiTZDateTime>>>) -> Self {
        Self(data)
    }
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&Url, &Vec<Region<FutureOrCurrentNairobiTZDateTime>>)> {
        self.0.iter()
    }
}
