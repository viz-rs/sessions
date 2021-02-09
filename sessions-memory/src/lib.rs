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
        let state = self.read()?.get(key).cloned();
        if let Some(State(time, data)) = state {
            if time >= Instant::now() {
                return Ok(Some(data));
            } else {
                self.remove(key).await?;
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use futures_executor::block_on;

    use sessions_core::*;

    use super::MemoryStorage;

    #[test]
    fn memory() -> Result<()> {
        block_on(async {
            let storage = Arc::new(MemoryStorage::default());

            let config = Arc::new(Config {
                cookie: CookieOptions::new(),
                storage: storage.clone(),
                generate: Box::new(|| nanoid::nanoid!(32)),
                verify: Box::new(|sid: &str| sid.len() == 32),
            });

            let id = config.generate();

            let session = Session::new(&id, 0, config.clone());

            assert_eq!(session.set::<String>("crate", "sessions".to_string()), None);

            assert!(session.save().await.is_ok());

            assert_eq!(session.get("crate"), Some("sessions".to_string()));

            assert_eq!(
                session.remove::<String>("crate"),
                Some("sessions".to_string())
            );

            assert_eq!(session.remove::<String>("crate"), None);

            assert_eq!(session.get::<String>("crate"), None);

            assert!(session.clear().is_ok());

            let mut session = Session::new(&id, 0, config.clone());

            if let Some(data) = storage.get(&id).await? {
                session.set_data(data)?;
            }

            assert_eq!(session.get("crate"), Some("sessions".to_string()));

            assert!(session.renew().await.is_ok());

            assert_ne!(id, session.id()?);

            assert!(session.destroy().await.is_ok());

            Ok(())
        })
    }
}
