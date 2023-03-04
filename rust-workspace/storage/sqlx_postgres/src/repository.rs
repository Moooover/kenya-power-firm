use crate::configuration::Settings;
use anyhow::Context;
use sqlx::postgres::PgPool;
use sqlx::{Connection, Executor, PgConnection};
use std::sync::Arc;
use uuid::Uuid;

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
        let pg_connection = Settings::with_db()?;
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
        let connection_options = Settings::without_db().unwrap().0;

        let mut connection = PgConnection::connect_with(&connection_options)
            .await
            .expect("Failed to connect to Postgres");

        let db_name = Uuid::new_v4();
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
            .await
            .expect("Failed to create database.");
        println!("The db name is {db_name}");

        let connection_with_db_name = connection_options.database(&db_name.to_string());

        // Migrate database
        let connection_pool = PgPool::connect_with(connection_with_db_name)
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!()
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        Self {
            pg_pool: Arc::new(connection_pool),
        }
    }
}
