use async_trait::async_trait;
use std::error::Error;
use web_page_extractor::text_extractor::{Area, TextExtractor};
struct Extractor {}

#[async_trait]
impl TextExtractor for Extractor {
    async fn extract(&self, text: String) -> Result<Vec<Area>, Box<dyn Error>> {
        todo!()
    }
}
