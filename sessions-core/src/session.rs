use std::{
    fmt,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, RwLock,
    },
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Value};

use crate::{Data, Error, CHANGED, PURGED, RENEWED, UNCHANGED};

/// The Session State
#[derive(Debug, Default)]
struct SessionState {
    status: AtomicU8,
    data: RwLock<Data>,
}

/// Session
#[derive(Clone)]
pub struct Session {
    state: Arc<SessionState>,
}

impl Session {
    /// Creates new `Session` with `Data`
    pub fn new(data: Data) -> Self {
        Self {
            state: Arc::new(SessionState {
                status: AtomicU8::new(UNCHANGED),
                data: RwLock::new(data),
            }),
        }
    }

    /// Gets status of the session
    pub fn status(&self) -> &AtomicU8 {
        &self.state.status
    }

    /// Gets lock data of the session
    pub fn lock_data(&self) -> &RwLock<Data> {
        &self.state.data
    }

    /// Gets a value by the key
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
    {
        match self
            .lock_data()
            .read()
            .map_err(|e| Error::RwLock(e.to_string()))?
            .get(key)
            .cloned()
        {
            Some(t) => from_value(t).map(Some).map_err(Error::Json),
            None => Ok(None),
        }
    }

    /// Sets a value by the key
    pub fn set<T>(&self, key: &str, val: T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            if let Ok(mut d) = self.lock_data().write() {
                // not allowed `RENEWED & CHANGED`
                if status == UNCHANGED {
                    self.status().store(CHANGED, Ordering::SeqCst);
                }
                d.insert(key.into(), to_value(val)?);
            }
        }
        Ok(())
    }

    /// Removes a value
    pub fn remove(&self, key: &str) -> Option<Value> {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            if let Ok(mut d) = self.lock_data().write() {
                // not allowed `RENEWED & CHANGED`
                if status == UNCHANGED {
                    self.status().store(CHANGED, Ordering::SeqCst);
                }
                return d.remove(key);
            }
        }
        None
    }

    /// Removes a value and deserialize
    pub fn remove_as<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.remove(key).and_then(|t| from_value(t).ok())
    }

    /// Clears the state
    pub fn clear(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            if let Ok(mut d) = self.lock_data().write() {
                // not allowed `RENEWED & CHANGED`
                if status == UNCHANGED {
                    self.status().store(CHANGED, Ordering::SeqCst);
                }
                d.clear();
            }
        }
    }

    /// Renews the new state
    pub fn renew(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED & RENEWED`
        if status != PURGED && status != RENEWED {
            self.status().store(RENEWED, Ordering::SeqCst)
        }
    }

    /// Destroys the current state from store
    pub fn purge(&self) {
        let status = self.status().load(Ordering::Acquire);
        // not allowed `PURGED`
        if status != PURGED {
            self.status().store(PURGED, Ordering::SeqCst);
            if let Ok(mut d) = self.lock_data().write() {
                d.clear();
            }
        }
    }

    /// Gets all raw key-value data from the session
    pub fn data(&self) -> Result<Data, Error> {
        self.lock_data()
            .read()
            .map_err(|e| Error::RwLock(e.to_string()))
            .map(|d| d.clone())
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.state.fmt(f)
    }
}
