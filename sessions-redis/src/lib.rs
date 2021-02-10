use std::time::Duration;

use sessions_core::{anyhow, async_trait, Data, Result, Storage};

use redis::{aio::Connection, AsyncCommands};

pub use redis::Client;

#[derive(Clone, Debug)]
pub struct RedisStorage {
    inner: Client,
}

impl RedisStorage {
    pub fn new(client: Client) -> Self {
        Self { inner: client }
    }

    pub async fn con(&self) -> Result<Connection> {
        self.inner
            .get_async_connection()
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }
}

#[async_trait]
impl Storage for RedisStorage {
    async fn get(&self, key: &str) -> Result<Option<Data>> {
        Ok(serde_json::from_slice(
            &self
                .con()
                .await?
                .get::<&str, Vec<u8>>(key)
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        )
        .ok())
    }

    async fn set(&self, key: &str, val: Data, exp: Duration) -> Result<()> {
        self.con()
            .await?
            .set_ex(key, serde_json::to_vec(&val)?, exp.as_secs() as usize)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }

    async fn remove(&self, key: &str) -> Result<()> {
        self.con()
            .await?
            .del(key)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }

    async fn reset(&self) -> Result<()> {
        redis::cmd("FLASHDB")
            .query_async(&mut self.con().await?)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }
}
