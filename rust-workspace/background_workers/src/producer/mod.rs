pub mod contracts;

use crate::app;
use celery::Celery;
use std::sync::Arc;

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
