use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use sessions_core::{Data, Storage};

#[derive(Debug, Clone)]
pub struct State(Instant, Data);

impl State {
    fn new(i: Instant, d: Data) -> Self {
        Self(i, d)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStorage {
    inner: Arc<RwLock<HashMap<String, State>>>,
}

impl MemoryStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a reference to the underlying data.
    #[must_use]
    pub fn get_ref(&self) -> &RwLock<HashMap<String, State>> {
        &self.inner
    }
}

impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<Data>> {
        let state = self
            .get_ref()
            .read()
            .map_err(into_io_error)?
            .get(key)
            .cloned();

        if let Some(State(time, data)) = state {
            if time >= Instant::now() {
                return Ok(Some(data));
            }
            self.remove(key).await?;
        }

        Ok(None)
    }

    async fn set(&self, key: &str, val: Data, exp: &Duration) -> Result<()> {
        self.get_ref()
            .write()
            .map_err(into_io_error)?
            .insert(key.to_string(), State::new(Instant::now() + *exp, val));
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<()> {
        self.get_ref().write().map_err(into_io_error)?.remove(key);
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        self.get_ref().write().map_err(into_io_error)?.clear();
        Ok(())
    }
}

#[inline]
fn into_io_error<E: std::error::Error>(e: E) -> Error {
    Error::new(ErrorKind::Other, e.to_string())
}
