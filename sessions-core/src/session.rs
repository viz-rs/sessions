use std::{
    fmt,
    io::{Error, ErrorKind, Result},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, RwLock,
    },
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{Data, State, CHANGED, PURGED, RENEWED, UNCHANGED};

/// Session
#[derive(Clone)]
pub struct Session {
    state: Arc<State>,
}

impl Session {
    /// Creates new `Session` with `Data`
    pub fn new(data: Data) -> Self {
        Self {
            state: Arc::new(State {
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
    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match self
            .lock_data()
            .read()
            .map_err(into_io_error)?
            .get(key)
            .cloned()
        {
            Some(t) => serde_json::from_value(t).map(Some).map_err(Into::into),
            None => Ok(None),
        }
    }

    /// Sets a value by the key
    pub fn set<T>(&self, key: &str, val: T) -> Result<()>
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
                d.insert(
                    key.into(),
                    serde_json::to_value(val).map_err(into_io_error)?,
                );
            }
        }
        Ok(())
    }

    /// Removes a value
    pub fn remove(&self, key: &str) -> Option<serde_json::Value> {
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
        serde_json::from_value(self.remove(key)?).ok()
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
    pub fn data(&self) -> Result<Data> {
        self.lock_data()
            .read()
            .map_err(into_io_error)
            .map(|d| d.clone())
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.state.fmt(f)
    }
}

#[inline]
fn into_io_error<E: std::error::Error>(e: E) -> Error {
    Error::new(ErrorKind::Other, e.to_string())
}
