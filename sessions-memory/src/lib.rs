use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use sessions_core::{async_trait, Data, Error, Storage};

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
    pub fn new() -> Self {
        Self {
            inner: Arc::default(),
        }
    }

    pub fn data(&self) -> &RwLock<HashMap<String, State>> {
        &self.inner
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<Data>, Error> {
        let state = self
            .data()
            .read()
            .map_err(|e| Error::RwLock(e.to_string()))?
            .get(key)
            .cloned();
        if let Some(State(time, data)) = state {
            if time >= Instant::now() {
                return Ok(Some(data));
            } else {
                self.remove(key).await?;
            }
        }

        Ok(None)
    }

    async fn set(&self, key: &str, val: Data, exp: &Duration) -> Result<(), Error> {
        self.data()
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?
            .insert(key.to_string(), State::new(Instant::now() + *exp, val));
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<(), Error> {
        self.data()
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?
            .remove(key);
        Ok(())
    }

    async fn reset(&self) -> Result<(), Error> {
        self.data()
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?
            .clear();
        Ok(())
    }
}
