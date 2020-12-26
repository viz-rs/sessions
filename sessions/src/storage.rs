use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;

use crate::Session;

#[async_trait]
pub trait Storage: Debug + Send + Sync + 'static {
    /// Get a data from storage by the key
    async fn get<S, G, V>(&self, key: &str) -> Result<Session<S, G, V>>;

    /// Set a data to storage by the key
    async fn set(&self, key: &str) -> Result<()>;

    /// Remove a data from storage by the key
    async fn remove<S, G, V>(&self, key: &str) -> Result<Session<S, G, V>>;

    /// Reset the storage and remove all keys
    async fn reset(&self) -> Result<()>;

    /// Close the connection
    async fn close(&self) -> Result<()>;
}
