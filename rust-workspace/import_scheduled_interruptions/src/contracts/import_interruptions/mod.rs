use crate::contracts::import_interruptions::db_access::ImportPowerInterruptionsDbAccess;
use crate::pdf_reader::ImportInput;
use crate::pdf_reader::PdfReader;
use crate::web_page_reader::WebPageReader;

mod db_access;

pub struct ImportInterruptions;

impl ImportInterruptions {
    pub async fn import() -> anyhow::Result<ImportInput> {
        let db_access = ImportPowerInterruptionsDbAccess::new();
        let web_page_reader = WebPageReader::new();
        let links = web_page_reader.fetch_links().await?;
        let pdf_reader = PdfReader::new();
        let extracted_data = pdf_reader.extract(links).await?;
        let input = ImportInput::new(extracted_data);
        db_access.import(&input).await?;
        Ok(input)
    }
}
