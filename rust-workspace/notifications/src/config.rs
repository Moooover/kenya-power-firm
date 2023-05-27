use lazy_static::lazy_static;
use secrecy::Secret;
use serde::Deserialize;
use shared_kernel::configuration::config;

#[derive(Deserialize)]
pub struct PoolSettings {
    pub notification_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub host: String,
    pub auth_token: Secret<String>,
    pub template_id: String,
}

#[derive(Deserialize)]
pub struct Settings {
    pub database: PoolSettings,
    pub email: EmailConfig,
}

lazy_static! {
    pub static ref SETTINGS_CONFIG: Settings = config::<Settings>().unwrap();
}
