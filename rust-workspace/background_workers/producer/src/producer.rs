use celery::Celery;
use std::sync::Arc;
use tasks::app;

#[derive(Clone)]
pub struct Producer {
    pub(crate) app: Arc<Celery>,
}

impl Producer {
    pub async fn new() -> anyhow::Result<Self> {
        let app = app().await?;

        Ok(Self { app })
    }
}
