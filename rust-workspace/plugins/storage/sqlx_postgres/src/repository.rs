use anyhow::anyhow;
use anyhow::Context;
use lazy_static::lazy_static;
use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

lazy_static! {
    static ref DB_URL: String = "postgres://postgres:postgres@localhost/blackout".to_string();
}

#[derive(Clone)]
pub struct Repository {
    pg_pool: Arc<PgPool>,
}

impl Repository {
    pub fn pool(&self) -> &PgPool {
        self.pg_pool.as_ref()
    }
    pub async fn new() -> anyhow::Result<Self> {
        let pg_pool = PgPool::connect(&**DB_URL)
            .await
            .context("Failed to connect to DB")
            .map(Arc::new)?;

        sqlx::migrate!()
            .run(&*pg_pool)
            .await
            .context("Failed to run migration")?;

        Ok(Self { pg_pool })
    }
}
