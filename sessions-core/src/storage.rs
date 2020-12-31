use std::{fmt::Debug, time::Duration};

use crate::{async_trait, Data, Result};

#[async_trait]
pub trait Storage: Debug + Send + Sync + 'static {
    /// Get a data from storage by the key
    async fn get(&self, key: &str) -> Result<Option<Data>>;

    /// Set a session to storage
    async fn set(&self, key: &str, val: Data, exp: Duration) -> Result<()>;

    /// Remove a data from storage by the key
    async fn remove(&self, key: &str) -> Result<()>;

    /// Reset the storage and remove all keys
    async fn reset(&self) -> Result<()> {
        Ok(())
    }

    /// Close the connection
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
