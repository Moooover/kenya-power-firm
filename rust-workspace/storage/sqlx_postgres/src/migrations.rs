use crate::configuration::Settings;
use anyhow::Context;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct MigrationManager {
    pg_pool: Arc<PgPool>,
}

impl MigrationManager {
    pub fn pool(&self) -> &PgPool {
        self.pg_pool.as_ref()
    }
    pub async fn new() -> anyhow::Result<Self> {
        let pg_connection = Settings::with_db()?;
        let pg_pool = PgPool::connect_with(pg_connection)
            .await
            .context("Failed to connect to DB")
            .map(Arc::new)?;

        Ok(Self { pg_pool })
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn new_test_manager(pool: Arc<PgPool>) -> Self {
        Self { pg_pool: pool }
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        let pool = self.pool();
        sqlx::migrate!()
            .run(pool)
            .await
            .context("Failed to run migration")
    }
}
