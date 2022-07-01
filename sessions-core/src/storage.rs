use std::time::Duration;

use crate::{async_trait, Data, Error};

/// A Storage Trait
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a data from storage by the key
    async fn get(&self, key: &str) -> Result<Option<Data>, Error>;

    /// Set a session to storage
    async fn set(&self, key: &str, val: Data, exp: &Duration) -> Result<(), Error>;

    /// Remove a data from storage by the key
    async fn remove(&self, key: &str) -> Result<(), Error>;

    /// Reset the storage and remove all keys
    async fn reset(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Close the connection
    async fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}
