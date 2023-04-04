use async_trait::async_trait;
use pdf_text_parser::PDFContentExtractor;
use producer::producer::Producer;
use sqlx_postgres::repository::Repository;
use std::sync::Arc;
use use_cases::actor::{Actor, Permissions, SubscriberExternalId};
use use_cases::notifications::notify_subscribers::Notifier;
use web_page_extractor::{pdf_extractor::PdfExtractorImpl, WebPageExtractor};

struct ImportActor;

#[async_trait]
impl Actor for ImportActor {
    fn permissions(&self) -> Permissions {
        // TODO: Get the permissions from auth0;
        let permissions: Vec<String> = vec!["import:affected_regions".to_string()];

        permissions.as_slice().into()
    }

    fn external_id(&self) -> SubscriberExternalId {
        "MAIN_IMPORTER".to_string().try_into().unwrap()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let repo = Repository::new().await?;

    let content_extractor = PDFContentExtractor;

    let content_extractor = Arc::new(content_extractor);

    let pdf_extractor = PdfExtractorImpl::new(content_extractor);

    let repo = Arc::new(repo);

    let producer = Producer::new().await?;
    let producer = Arc::new(producer);

    let notification = Arc::new(Notifier::new(repo.clone(), producer));

    let importer =
        use_cases::import_affected_areas::ImportAffectedAreas::new(repo.clone(), notification);

    let importer = Arc::new(importer);

    let extractor = WebPageExtractor::new(importer, repo, Arc::new(pdf_extractor));
    extractor.run(&ImportActor {}).await?;

    Ok(())
}
