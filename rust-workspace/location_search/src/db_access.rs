#[cfg(not(test))]
use crate::config::SETTINGS_CONFIG;
use async_once::AsyncOnce;
use lazy_static::lazy_static;
use sqlx_postgres::pool_manager::{PoolManager, PoolWrapper};

#[cfg(not(test))]
lazy_static! {
    static ref POOL_MANAGER: AsyncOnce<PoolManager> = AsyncOnce::new(async {
        PoolManager::new(SETTINGS_CONFIG.database.search_connections)
            .await
            .expect("PoolManager not initialized")
    });
}

#[cfg(test)]
lazy_static! {
    static ref POOL_MANAGER: AsyncOnce<PoolManager> = AsyncOnce::new(async {
        PoolManager::new()
            .await
            .expect("PoolManager not initialized")
    });
}

#[derive(Copy, Clone)]
pub struct DbAccess;

impl DbAccess {
    #[cfg(not(test))]
    pub async fn pool(&self) -> PoolWrapper<'static> {
        POOL_MANAGER.get().await.pool()
    }

    #[cfg(test)]
    pub async fn pool(&self) -> PoolWrapper<'static> {
        POOL_MANAGER.get().await.pool()
    }
}
