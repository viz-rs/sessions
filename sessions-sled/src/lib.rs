use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::{Duration, Instant},
};

use sessions_core::{anyhow, async_trait, Data, Result, Storage};

#[derive(Clone, Debug)]
struct State(Instant, Data);

impl State {
    fn new(i: Instant, d: Data) -> Self {
        Self(i, d)
    }
}

#[derive(Debug, Default)]
pub struct MemoryStorage {
    inner: Arc<RwLock<HashMap<String, State>>>,
}

impl MemoryStorage {
    fn read(&self) -> Result<RwLockReadGuard<'_, HashMap<String, State>>> {
        self.inner.read().map_err(|e| anyhow!(e.to_string()))
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, HashMap<String, State>>> {
        self.inner.write().map_err(|e| anyhow!(e.to_string()))
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<Data>> {
        if let Some(State(time, data)) = self.read()?.get(key).cloned() {
            if time >= Instant::now() {
                return Ok(Some(data));
            }
        }

        Ok(None)
    }

    async fn set(&self, key: &str, val: Data, exp: Duration) -> Result<()> {
        self.write()?
            .insert(key.to_string(), State::new(Instant::now() + exp, val));
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<()> {
        self.write()?.remove(key);
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        Ok(self.write()?.clear())
    }
}
