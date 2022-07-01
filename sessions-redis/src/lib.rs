use std::{fmt::Debug, time::Duration};

use redis::{aio::ConnectionLike, AsyncCommands};

use sessions_core::{async_trait, Data, Error, Storage};

pub use redis::Client;

#[derive(Clone, Debug)]
pub struct RedisStorage<T> {
    inner: T,
}

impl<T> RedisStorage<T> {
    pub fn new(client: T) -> Self {
        Self { inner: client }
    }
}

#[async_trait]
impl<T> Storage for RedisStorage<T>
where
    T: ConnectionLike + Clone + Send + Sync,
{
    async fn get(&self, key: &str) -> Result<Option<Data>, Error> {
        Ok(serde_json::from_slice(
            &self
                .inner
                .clone()
                .get::<&str, Vec<u8>>(key)
                .await
                .map_err(|e| Error::Connection(e.to_string()))?,
        )
        .ok())
    }

    async fn set(&self, key: &str, val: Data, exp: &Duration) -> Result<(), Error> {
        self.inner
            .clone()
            .set_ex(key, serde_json::to_vec(&val)?, exp.as_secs() as usize)
            .await
            .map_err(|e| Error::Connection(e.to_string()))
    }

    async fn remove(&self, key: &str) -> Result<(), Error> {
        self.inner
            .clone()
            .del(key)
            .await
            .map_err(|e| Error::Connection(e.to_string()))
    }

    async fn reset(&self) -> Result<(), Error> {
        redis::cmd("FLASHDB")
            .query_async(&mut self.inner.clone())
            .await
            .map_err(|e| Error::Connection(e.to_string()))
    }
}
