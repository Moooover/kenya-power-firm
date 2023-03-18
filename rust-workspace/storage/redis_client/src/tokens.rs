use crate::client::Client;
use anyhow::Context;

impl Client {
    pub async fn get_token(&self, key: &str) -> anyhow::Result<usize> {
        let mut conn = self.conn.clone();
        let _ = redis::Cmd::decr(key, 1)
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to decr token count for key {key}"))?;
        redis::Cmd::get(key)
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to get token count for key {key}"))
    }
}
