use std::{fmt, sync::Arc, time::Duration};

use crate::{async_trait, CookieOptions, Data, Result, Storage};

pub struct Config {
    pub cookie: CookieOptions,
    pub storage: Arc<dyn Storage>,
    pub generate: Box<dyn GenerateFn>,
    pub verify: Box<dyn VerifyFn>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("cookie", &self.cookie)
            .field("storage", &self.storage)
            .finish()
    }
}

impl Config {
    pub fn storage(&self) -> Arc<dyn Storage> {
        self.storage.clone()
    }

    pub fn cookie(&self) -> &CookieOptions {
        &self.cookie
    }

    /// Generate a session id
    pub fn generate(&self) -> String {
        self.generate.call()
    }

    /// Verify a session id
    pub fn verify(&self, key: &str) -> bool {
        self.verify.call(key)
    }

    pub fn max_age(&self) -> Duration {
        self.cookie.max_age
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

pub trait GenerateFn
where
    Self: Send + Sync + 'static,
{
    fn call(&self) -> String;
}

pub trait VerifyFn
where
    Self: Send + Sync + 'static,
{
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
