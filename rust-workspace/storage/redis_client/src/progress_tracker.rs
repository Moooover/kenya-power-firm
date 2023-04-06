use crate::client::Client;
use anyhow::Context;
use redis::{FromRedisValue, ToRedisArgs};
use std::fmt::Display;

impl Client {
    pub async fn set_status<K, V>(&self, key: K, value: V) -> anyhow::Result<V>
    where
        K: Display + Clone + ToRedisArgs,
        V: FromRedisValue + ToRedisArgs,
    {
        let mut conn = self.conn.clone();

        redis::Cmd::set_ex(key.clone(), value, 1200)
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to set status for key {key}"))?;

        redis::Cmd::get(key.clone())
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to get status for key {key}"))
    }

    pub async fn get_status<K, V>(&self, key: K) -> anyhow::Result<Option<V>>
    where
        K: Display + Clone + ToRedisArgs,
        V: FromRedisValue,
    {
        let mut conn = self.conn.clone();

        redis::Cmd::get(key.clone())
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to get status for key {key}"))
    }
}
