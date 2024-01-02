use std::{
    fmt::Debug,
    io::{Error, ErrorKind, Result},
    time::Duration,
};

use redis::{aio::ConnectionLike, AsyncCommands};
use sessions_core::{Data, Storage};

#[derive(Clone, Debug)]
pub struct RedisStorage<T> {
    inner: T,
}

impl<T> RedisStorage<T> {
    #[must_use]
    pub fn new(client: T) -> Self {
        Self { inner: client }
    }

    /// Gets a reference to the underlying client.
    #[must_use]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> Storage for RedisStorage<T>
where
    T: ConnectionLike + Clone + Send + Sync,
{
    async fn get(&self, key: &str) -> Result<Option<Data>> {
        Ok(serde_json::from_slice(
            &self
                .get_ref()
                .clone()
                .get::<&str, Vec<u8>>(key)
                .await
                .map_err(into_io_error)?,
        )
        .ok())
    }

    async fn set(&self, key: &str, val: Data, exp: &Duration) -> Result<()> {
        self.get_ref()
            .clone()
            .set_ex(key, serde_json::to_vec(&val)?, exp.as_secs())
            .await
            .map_err(into_io_error)
    }

    async fn remove(&self, key: &str) -> Result<()> {
        self.get_ref().clone().del(key).await.map_err(into_io_error)
    }

    async fn reset(&self) -> Result<()> {
        redis::cmd("FLASHDB")
            .query_async(&mut self.get_ref().clone())
            .await
            .map_err(into_io_error)
    }
}

#[inline]
fn into_io_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
    Error::new(ErrorKind::Other, e)
}
