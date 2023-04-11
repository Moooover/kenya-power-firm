use crate::client::Client;
use anyhow::Context;
use redis::{FromRedisValue, ToRedisArgs};
use std::fmt::Display;

const EXPIRY_TIME_IN_SECONDS: usize = 1200;

impl Client {
    pub async fn set_status<K, V>(&self, key: K, value: V) -> anyhow::Result<V>
    where
        K: Display + Clone + ToRedisArgs,
        V: FromRedisValue + ToRedisArgs,
    {
        let mut conn = self.conn.clone();

        let (v,): (V,) = redis::pipe()
            .atomic()
            .set_ex(key.clone(), value, EXPIRY_TIME_IN_SECONDS)
            .ignore()
            .get(key.clone())
            .query_async(&mut conn)
            .await
            .with_context(|| format!("Failed to get and set value for key {key}"))?;

        Ok(v)
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
