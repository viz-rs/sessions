use std::{fmt, sync::Arc, time::Duration};

use crate::{async_trait, CookieOptions, Data, Result, Storage};

/// Sessions Config
pub struct Config {
    /// Cookie Options
    pub cookie: CookieOptions,
    /// Current Storage
    pub storage: Arc<dyn Storage>,
    /// Generates session id
    pub generate: Box<dyn Send + Sync + 'static + Fn() -> String>,
    /// Verifes session id
    pub verify: Box<dyn Send + Sync + 'static + Fn(&str) -> bool>,
}

impl Config {
    /// Gets current storage
    pub fn storage(&self) -> Arc<dyn Storage> {
        self.storage.clone()
    }

    /// Gets cookie options
    pub fn cookie(&self) -> &CookieOptions {
        &self.cookie
    }

    /// Gets cookie's max_age or session's expries
    pub fn max_age(&self) -> Duration {
        self.cookie.max_age
    }

    /// Generates a session id
    pub fn generate(&self) -> String {
        (self.generate)()
    }

    /// Verifes a session id
    pub fn verify(&self, key: &str) -> bool {
        (self.verify)(key)
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("cookie", &self.cookie)
            .field("storage", &self.storage)
            .finish()
    }
}

#[async_trait]
impl Storage for Config {
    /// Get a data from storage by the key
    async fn get(&self, key: &str) -> Result<Option<Data>> {
        self.storage.get(key).await
    }

    /// Set a data to storage by the key
    async fn set(&self, key: &str, val: Data, exp: Duration) -> Result<()> {
        self.storage.set(key, val, exp).await
    }

    /// Remove a data from storage by the key
    async fn remove(&self, key: &str) -> Result<()> {
        self.storage.remove(key).await
    }

    /// Reset the storage and remove all keys
    async fn reset(&self) -> Result<()> {
        self.storage.reset().await
    }

    /// Close the connection
    async fn close(&self) -> Result<()> {
        self.storage.close().await
    }
}
