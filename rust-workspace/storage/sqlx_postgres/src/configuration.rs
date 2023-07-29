use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use shared_kernel::configuration::config;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Debug, Deserialize)]
pub struct Settings {
    database: DbSettings,
}

type DbName = String;
#[derive(Debug, Deserialize)]
pub struct DbSettings {
    host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    port: u16,
    username: String,
    password: Secret<String>,
    database_name: DbName,
    require_ssl: bool,
}

impl Settings {
    fn parse() -> anyhow::Result<Self> {
        config::<Settings>()
    }
    pub fn without_db() -> anyhow::Result<(PgConnectOptions, DbName)> {
        let config = Self::parse()?.database;
        let ssl_mode = if config.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        Ok((
            PgConnectOptions::new()
                .host(&config.host)
                .username(&config.username)
                .password(config.password.expose_secret())
                .port(config.port)
                .ssl_mode(ssl_mode),
            config.database_name,
        ))
    }

    pub fn with_db() -> anyhow::Result<PgConnectOptions> {
        let (options, database_name) = Self::without_db()?;
        Ok(options.database(&database_name))
    }

    
}
