pub mod pdf_extractor;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use anyhow::Context;
use async_trait::async_trait;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use use_cases::actor::Actor;
use use_cases::import_planned_blackouts::{
    Area, ImportInput, ImportPlannedBlackoutsInteractor, Region, Url,
};

use regex::Regex;

lazy_static! {
    static ref CLIENT: ClientWithMiddleware =   {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(reqwest::Client::new())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()

    };
}

struct WebPageExtractor {
    importer: Arc<dyn ImportPlannedBlackoutsInteractor>,
    file_operations: Arc<dyn FileOperations>,
    pdf_reader: Arc<dyn PdfExtractor>,
}

#[async_trait]
pub trait FileOperations: Send + Sync {
    async fn save_and_return_unprocessed_files(&self, files: Vec<Url>) -> anyhow::Result<Vec<Url>>;
}

#[async_trait]
pub trait PdfExtractor: Send + Sync {
    async fn extract(&self, links: Vec<Url>) -> anyhow::Result<HashMap<Url, Vec<Region>>>;
}

impl WebPageExtractor {
    pub async fn run(&self, actor: &dyn Actor) -> anyhow::Result<()> {
        let pdf_links = get_pdf_links().await?;

        let unprocessed_files = self
            .file_operations
            .save_and_return_unprocessed_files(pdf_links)
            .await?;

        let result = self.pdf_reader.extract(unprocessed_files).await?;

        self.importer.import(actor, ImportInput(result)).await
    }
}

async fn get_page_contents() -> anyhow::Result<String> {
    CLIENT
        .get("https://kplc.co.ke/category/view/50/planned-power-interruptions")
        .send()
        .await?
        .text()
        .await
        .context("Failed to read web page")
}

async fn get_pdf_links() -> anyhow::Result<Vec<Url>> {
    lazy_static! {
        static ref PDF_LINKS_REGEX: Regex =
            Regex::new(r"https://www\.kplc\.co\.ke/img/full/.*\.pdf")
                .expect("PDF_LINKS_REGEX to compile");
    }

    let page_content = get_page_contents().await?;

    Ok(PDF_LINKS_REGEX
        .find_iter(&page_content)
        .into_iter()
        .map(|a_match| Url(a_match.as_str().to_string()))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::{get_page_contents, get_pdf_links};

    #[tokio::test]
    async fn test_get_page_contents() {
        get_page_contents().await;
    }

    #[tokio::test]
    async fn test_links() {
        let links = get_pdf_links().await.unwrap();
        println!("{links:?}")
    }
}
