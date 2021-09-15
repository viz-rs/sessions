use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
    time::Duration,
};

use crate::{
    anyhow,
    data::{from_value, to_value, DeserializeOwned, Serialize},
    Config, Data, Result, Storage,
};

/// Session
#[derive(Clone)]
pub struct Session<G, V> {
    /// Session's Config
    config: Arc<Config<G, V>>,
    /// Session's status, 0: inited, 1: saved, 2: renewed, 3: destroyed
    status: Arc<AtomicUsize>,
    /// Session's Data status, false: unchanged, true: changed
    data_status: Arc<AtomicBool>,
    /// Session's `SessionBeer`
    beer: Arc<RwLock<SessionBeer>>,
}

impl<G, V> Session<G, V>
where
    G: Send + Sync + 'static + Fn() -> String,
    V: Send + Sync + 'static + Fn(&str) -> bool,
{
    /// Creates new `Session` with `id` `status` and `Config`
    pub fn new(id: &str, status: usize, config: Arc<Config<G, V>>) -> Self {
        Self {
            config,
            status: Arc::new(AtomicUsize::new(status)),
            data_status: Arc::new(AtomicBool::new(false)),
            beer: Arc::new(RwLock::new(SessionBeer {
                id: id.into(),
                data: Data::new(),
            })),
        }
    }

    /// Reads the session expires or cookie max_age
    pub fn max_age(&self) -> Duration {
        self.config.max_age()
    }

    /// Reads the session beer
    pub fn beer(&self) -> Result<RwLockReadGuard<'_, SessionBeer>> {
        self.beer.read().map_err(|e| anyhow!(e.to_string()))
    }

    /// Writes the session beer
    pub fn beer_mut(&self) -> Result<RwLockWriteGuard<'_, SessionBeer>> {
        self.beer.write().map_err(|e| anyhow!(e.to_string()))
    }

    /// Reads the session state
    pub fn data(&self) -> Result<Data> {
        Ok(self.beer()?.data.clone())
    }

    /// Writes the session state
    pub fn set_data(&self, data: Data) -> Result<()> {
        self.beer_mut()?.data = data;
        Ok(())
    }

    /// Gets the session id
    pub fn id(&self) -> Result<String> {
        Ok(self.beer()?.id.clone())
    }

    /// Gets the session id
    pub fn set_id(&self, id: &str) -> Result<()> {
        self.beer_mut()?.id = id.into();
        Ok(())
    }

    /// Gets the session data status
    pub fn data_status(&self) -> bool {
        self.data_status.load(Ordering::Relaxed)
    }

    /// Gets the session status
    pub fn status(&self) -> usize {
        self.status.load(Ordering::Relaxed)
    }

    /// Gets a value by the key
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        from_value(self.data().ok()?.get(key).cloned()?).ok()
    }

    /// Sets a value by the key
    pub fn set<T: DeserializeOwned + Serialize>(&self, key: &str, val: T) -> Option<T> {
        let prev = self
            .beer_mut()
            .ok()?
            .data
            .insert(key.into(), to_value(val).ok()?);
        self.data_status.store(true, Ordering::SeqCst);
        from_value(prev?).ok()
    }

    /// Removes a value
    pub fn remove<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let prev = self.beer_mut().ok()?.data.remove(key)?;
        self.data_status.store(true, Ordering::SeqCst);
        from_value(prev).ok()
    }

    /// Clears the state
    pub fn clear(&self) -> Result<()> {
        self.beer_mut()?.data.clear();
        self.data_status.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Saves the current state to the store
    pub async fn save(&self) -> Result<()> {
        if self.status.fetch_add(1, Ordering::SeqCst) == 0 {
            self.config
                .set(&self.id()?, self.data()?.clone(), self.max_age())
                .await?;
        }
        Ok(())
    }

    /// Renews the new state
    pub async fn renew(&mut self) -> Result<()> {
        if self.status.load(Ordering::Relaxed) < 2 {
            self.config.remove(&self.id()?).await?;
            self.beer_mut()?.data.clear();
            self.set_id(&self.config.generate())?;
            self.config
                .set(&self.id()?, self.data()?, self.max_age())
                .await?;
            self.status.store(2, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Destroys the current state from store
    pub async fn destroy(&self) -> Result<()> {
        if self.status.load(Ordering::Relaxed) < 3 {
            self.config.remove(&self.id()?).await?;
            self.status.store(3, Ordering::SeqCst);
        }
        Ok(())
    }
}

impl<G, V> fmt::Debug for Session<G, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("status", &self.status)
            .field("data_status", &self.data_status)
            .field("beer", &self.beer)
            .field("config", &self.config)
            .finish()
    }
}

/// A Session Beer
#[derive(Debug, Clone, Default)]
pub struct SessionBeer {
    /// Session's id
    pub id: String,
    /// Session's Data
    pub data: Data,
}
