use crate::configuration::DbSettings;
use anyhow::anyhow;
use anyhow::Context;
use lazy_static::lazy_static;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPool};
use sqlx::postgres::{PgPoolOptions, PgSslMode};
use sqlx::{Connection, Executor, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

lazy_static! {
    static ref DB_URL: String = "postgres://postgres:postgres@localhost/blackout".to_string();
}

#[derive(Clone)]
pub struct Repository {
    pg_pool: Arc<PgPool>,
}

impl Repository {
    pub fn pool(&self) -> &PgPool {
        self.pg_pool.as_ref()
    }
    #[cfg(not(test))]
    pub async fn new() -> anyhow::Result<Self> {
        let pg_connection = DbSettings::with_db()?;
        let pg_pool = PgPool::connect_with(pg_connection)
            .await
            .context("Failed to connect to DB")
            .map(Arc::new)?;

        sqlx::migrate!()
            .run(&*pg_pool)
            .await
            .context("Failed to run migration")?;

        Ok(Self { pg_pool })
    }
    #[cfg(test)]
    pub async fn new_test_repo() -> Self {
        let mut connection_options = DbSettings::without_db().unwrap().0;

        let mut connection = PgConnection::connect_with(&connection_options)
            .await
            .expect("Failed to connect to Postgres");

        let db_name = Uuid::new_v4();
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
            .await
            .expect("Failed to create database.");

        let connection_with_db_name = connection_options.database(&db_name.to_string());

        // Migrate database
        let connection_pool = PgPool::connect_with(connection_with_db_name)
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        Self {
            pg_pool: Arc::new(connection_pool),
        }
    }
}
