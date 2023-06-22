use anyhow::Context;
use lazy_static::lazy_static;
use secrecy::Secret;
use serde::Deserialize;
use shared_kernel::configuration::config;

#[derive(Debug, Deserialize, Clone)]
pub struct LocationSearcherConfig {
    pub host: String,
    pub api_key: Secret<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub host: String,
    pub auth_token: Secret<String>,
}

#[derive(Debug, Deserialize)]
pub struct RedisSettings {
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct ExternalApiRateLimits {
    pub email: usize,
    pub location: usize,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub location: LocationSearcherConfig,
    pub email: EmailConfig,
    pub redis: RedisSettings,
    pub external_api_rate_limits: ExternalApiRateLimits,
}

impl Settings {
    pub fn parse() -> anyhow::Result<Settings> {
        config::<Settings>().context("Failed to deserialize settings to background_worker settings")
    }
}

lazy_static! {
    pub static ref SETTINGS_CONFIG: Settings = Settings::parse().unwrap();
}
