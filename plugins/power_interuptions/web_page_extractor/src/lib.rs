pub mod pdf_extractor;

use anyhow::Context;
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use use_cases::import_planned_blackouts::{
    Area, ImportInput, ImportPlannedBlackoutsInteractor, Region, Url,
};

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
    pub async fn run(&self) -> anyhow::Result<()> {
        let page_content = get_page_contents().await?;
        let pdf_links = get_pdf_links(page_content);

        let unprocessed_files = self
            .file_operations
            .save_and_return_unprocessed_files(pdf_links)
            .await?;

        let result = self.pdf_reader.extract(unprocessed_files).await?;

        self.importer.import(ImportInput(result)).await
    }
}

async fn get_page_contents() -> anyhow::Result<String> {
    reqwest::get("https://kplc.co.ke/category/view/50/planned-power-interruptions")
        .await?
        .text()
        .await
        .context("Failed to read web page")
}

fn get_pdf_links(content: String) -> Vec<Url> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::get_page_contents;

    #[tokio::test]
    async fn test_get_page_contents() {
        get_page_contents().await;
    }
}
