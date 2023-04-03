use async_trait::async_trait;
use job_runners::alert_email_sender::send_alert;
use pdf_text_parser::PDFContentExtractor;
use std::sync::Arc;
use url::Url;
use use_cases::{
    actor::{Actor, Permissions, SubscriberExternalId},
    import_affected_areas::{ImportInput, ImportPlannedBlackoutsInteractor},
};
use web_page_extractor::FileOperations;
use web_page_extractor::{pdf_extractor::PdfExtractorImpl, WebPageExtractor};

struct DryRunFileOps;

#[async_trait]
impl FileOperations for DryRunFileOps {
    async fn save_files(&self, _files: Vec<Url>) -> anyhow::Result<()> {
        Ok(())
    }
    async fn return_unprocessed_files(&self, files: Vec<Url>) -> anyhow::Result<Vec<Url>> {
        // We can assume that the first item in the list is the newest entry and so we return it as unprocessed;
        let files = files.first().map(|url| vec![url.clone()]).unwrap_or(files);
        Ok(files)
    }
}

struct DryRunImportInteractor;

#[async_trait]
impl ImportPlannedBlackoutsInteractor for DryRunImportInteractor {
    async fn import(&self, _actor: &dyn Actor, _data: ImportInput) -> anyhow::Result<()> {
        Ok(())
    }
}

struct DryRunActor;

#[async_trait]
impl Actor for DryRunActor {
    fn permissions(&self) -> Permissions {
        let permissions: Vec<String> = vec![];

        permissions.as_slice().into()
    }

    fn external_id(&self) -> SubscriberExternalId {
        "DRY_RUN".to_string().try_into().unwrap()
    }
}

#[tokio::main]
async fn main() {
    let content_extractor = PDFContentExtractor;

    let content_extractor = Arc::new(content_extractor);

    let pdf_extractor = PdfExtractorImpl::new(content_extractor);

    let file_ops = Arc::new(DryRunFileOps {});

    let importer = Arc::new(DryRunImportInteractor {});

    let extractor = WebPageExtractor::new(importer, file_ops, Arc::new(pdf_extractor));

    if let Err(err) = extractor.run(&DryRunActor {}).await {
        println!("{err:?}");
        if let Err(err) = send_alert(err).await {
            println!("{}", err)
        }
    }
}
