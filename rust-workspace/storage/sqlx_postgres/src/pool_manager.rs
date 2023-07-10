use crate::configuration::Settings;
#[cfg(test)]
use crate::migrations::MigrationManager;
#[cfg(test)]
use sqlx::Executor;

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

#[derive(Clone)]
pub struct PoolManager {
    pub(crate) pg_pool: Arc<PgPool>,
}

#[derive(Clone)]
pub struct PoolWrapper<'a>(&'a PgPool);

impl<'a> AsRef<PgPool> for PoolWrapper<'a> {
    fn as_ref(&self) -> &'a PgPool {
        self.0
    }
}

impl PoolManager {
    #[cfg(not(test))]
    pub async fn new(max_connections: u32) -> anyhow::Result<Self> {
        let pg_connection = Settings::with_db()?;

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(pg_connection)
            .await?;
        Ok(Self {
            pg_pool: Arc::new(pool),
        })
    }

    pub fn pool(&self) -> PoolWrapper {
        PoolWrapper(self.pg_pool.as_ref())
    }

    #[cfg(test)]
    pub async fn new() -> anyhow::Result<Self> {
        let (options, _) = Settings::without_db()?;

        let pool = PgPoolOptions::new().connect_with(options.clone()).await?;
        let test_db_name = uuid::Uuid::new_v4();
        let _ = pool
            .execute(format!(r#"CREATE DATABASE "{}";"#, test_db_name).as_str())
            .await;
        let pool = PgPoolOptions::new()
            .connect_with(options.database(&test_db_name.to_string()))
            .await
            .map(Arc::new)?;

        let migration_manager = MigrationManager::new(Arc::clone(&pool));
        migration_manager.migrate().await?;

        Ok(Self { pg_pool: pool })
    }
}
