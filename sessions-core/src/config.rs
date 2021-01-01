use std::{fmt, sync::Arc, time::Duration};

use crate::{async_trait, CookieOptions, Data, Result, Storage};

/// Sessions Config
pub struct Config {
    /// Cookie Options
    pub cookie: CookieOptions,
    /// Current Storage
    pub storage: Arc<dyn Storage>,
    /// Generates session id
    pub generate: Box<dyn GenerateFn>,
    /// Verifes session id
    pub verify: Box<dyn VerifyFn>,
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
        self.generate.call()
    }

    /// Verifes a session id
    pub fn verify(&self, key: &str) -> bool {
        self.verify.call(key)
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

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("cookie", &self.cookie)
            .field("storage", &self.storage)
            .finish()
    }
}

/// A trait for generating session id
pub trait GenerateFn
where
    Self: Send + Sync + 'static,
{
    #[allow(missing_docs)]
    #[must_use]
    fn call(&self) -> String;
}

/// A trait for verifing session id
pub trait VerifyFn
where
    Self: Send + Sync + 'static,
{
    #[allow(missing_docs)]
    #[must_use]
    fn call(&self, key: &str) -> bool;
}

impl<F> GenerateFn for F
where
    F: Send + Sync + 'static + Fn() -> String,
{
    fn call(&self) -> String {
        (self)()
    }
}

impl<F> VerifyFn for F
where
    F: Send + Sync + 'static + Fn(&str) -> bool,
{
    fn call(&self, key: &str) -> bool {
        (self)(key)
    }
}
