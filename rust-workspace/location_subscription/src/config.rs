use lazy_static::lazy_static;
use secrecy::Secret;
use serde::Deserialize;
use shared_kernel::configuration::config;

#[derive(Deserialize)]
pub struct PoolSettings {
    pub location_connections: u32,
}

#[derive(Deserialize)]
pub struct Settings {
    pub database: PoolSettings,
    pub location: LocationSearcherConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocationSearcherConfig {
    pub host: String,
    pub api_key: Secret<String>,
}

lazy_static! {
    pub static ref SETTINGS_CONFIG: Settings = config::<Settings>().unwrap();
}
