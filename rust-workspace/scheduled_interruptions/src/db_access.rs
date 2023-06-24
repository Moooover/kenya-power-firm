use crate::config::SETTINGS_CONFIG;

use async_once::AsyncOnce;

use lazy_static::lazy_static;

use sqlx_postgres::pool_manager::{PoolManager, PoolWrapper};

lazy_static! {
    static ref POOL_MANAGER: AsyncOnce<PoolManager> = AsyncOnce::new(async {
        PoolManager::new(
            SETTINGS_CONFIG
                .database
                .import_scheduled_interrupts_connections,
        )
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
