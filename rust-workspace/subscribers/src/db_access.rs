use async_once::AsyncOnce;
use lazy_static::lazy_static;
use serde::Deserialize;
use shared_kernel::configuration::config;
use sqlx_postgres::pool_manager::{PoolManager, PoolWrapper};

#[derive(Deserialize)]
pub struct PoolSettings {
    pub subscriber_connections: u32,
}

#[derive(Deserialize)]
pub struct Settings {
    pub database: PoolSettings,
}

lazy_static! {
    pub static ref SETTINGS_CONFIG: Settings = config::<Settings>().unwrap();
}

lazy_static! {
    static ref POOL_MANAGER: AsyncOnce<PoolManager> = AsyncOnce::new(async {
        PoolManager::new(SETTINGS_CONFIG.database.subscriber_connections)
            .await
            .expect("PoolManager not initialized")
    });
}

#[derive(Copy, Clone)]
pub struct DbAccess;

impl DbAccess {
    pub async fn pool(&self) -> PoolWrapper<'static> {
        POOL_MANAGER.get().await.pool()
    }
}
