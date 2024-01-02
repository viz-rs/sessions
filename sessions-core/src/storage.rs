use std::{future::Future, io::Result, time::Duration};

use crate::Data;

/// A Storage Trait
pub trait Storage: Send + Sync {
    /// Gets a [`Data`] from storage by the key
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<Data>>> + Send;

    /// Sets a session [`Data`] into storage
    fn set(&self, key: &str, val: Data, exp: &Duration) -> impl Future<Output = Result<()>> + Send;

    /// Removes a data from storage by the key
    fn remove(&self, key: &str) -> impl Future<Output = Result<()>> + Send;

    /// Resets the storage and remove all keys
    fn reset(&self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Closes the connection
    fn close(&self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}
