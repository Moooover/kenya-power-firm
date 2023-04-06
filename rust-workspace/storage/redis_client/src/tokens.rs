use crate::client::Client;
use anyhow::Context;

impl Client {
    pub async fn decr_count(&self, key: &str, count: usize) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        redis::Cmd::decr(key, count)
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to decr token count for key {key}"))?;

        Ok(())
    }
    pub async fn get_token_count(&self, key: &str) -> anyhow::Result<i32> {
        self.decr_count(key, 1).await?;
        let mut conn = self.conn.clone();
        redis::Cmd::get(key)
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to get token count for key {key}"))
    }
}
