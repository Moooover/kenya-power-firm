use crate::PdfExtractor;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use use_cases::import_planned_blackouts::{Area, Region, Url};

lazy_static! {
    static ref FORWARD_SLASH: Regex =
        Regex::new("/").expect("Expected regex to compile forward slash pattern");
}

#[async_trait]
pub trait TextExtractor: Send + Sync {
    async fn extract(&self, text: String) -> anyhow::Result<Vec<Region>>;
}

struct PdfExtractorImpl;

#[async_trait]
impl PdfExtractor for PdfExtractorImpl {
    async fn extract(&self, links: Vec<Url>) -> anyhow::Result<HashMap<Url, Vec<Region>>> {
        for link in &links {
            let res = reqwest::get(&link.0)
                .await
                .context("Failed to fetch link")?
                .bytes()
                .await
                .context("Failed to convert to bytes")?;
            let text = resolve_text_from_file(&link.0, &res).await?;
            println!("{text}")
        }

        Ok(HashMap::new())
    }
}

async fn resolve_text_from_file(url: &str, file_bytes: &[u8]) -> anyhow::Result<String> {
    use pdf_extract::extract_text;
    use std::env;
    use tokio::fs::remove_file;
    use tokio::fs::File;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;

    let path = env::current_dir().context("Cannot read current_dir")?;
    let normalized_url = FORWARD_SLASH.replace_all(url, "_");
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
    use crate::pdf_extractor::PdfExtractorImpl;
    use crate::PdfExtractor;
    use use_cases::import_planned_blackouts::Url;

    #[tokio::test]
    async fn test_working() {
        let extractor = PdfExtractorImpl;
        let links = vec![Url(
            "https://www.kplc.co.ke/img/Interruptions%20-%2009.02.2023.pdf".to_owned(),
        )];
        let res = extractor.extract(links).await.expect("Expected result");
    }
}
