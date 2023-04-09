use anyhow::Context;
use sqlx_postgres::repository::Repository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let repo = Repository::new().await?;
    let pool = repo.pool();
    sqlx::migrate!()
        .run(pool)
        .await
        .context("Failed to run migration")
}
