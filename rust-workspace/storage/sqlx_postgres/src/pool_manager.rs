use crate::configuration::Settings;
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
}
