use anyhow::Context;
use tasks::app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app().await?;

    app.consume_from(&["locations_queue", "celery", "email_notifications_queue"])
        .await
        .context("Failed to consume tasks")
}
