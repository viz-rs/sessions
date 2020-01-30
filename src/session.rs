use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Map, Value};
use std::{
    error::Error as ErrorExt,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub trait Sessionable {
    fn state(&self) -> Result<RwLockReadGuard<'_, Map<String, Value>>, Error>;
    fn state_mut(&self) -> Result<RwLockWriteGuard<'_, Map<String, Value>>, Error>;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;
    fn set<T: DeserializeOwned + Serialize>(&self, key: &str, val: T) -> Result<Option<T>, Error>;
    fn remove<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;
    fn clear(&self) -> Result<(), Error>;
}

#[derive(Debug, Default)]
pub struct Session {
    /// Why not use `Rc<RefCell<Map<String, Value>>>`?
    /// See: https://github.com/hyperium/http/blob/master/src/extensions.rs
    state: Arc<RwLock<Map<String, Value>>>,
}

impl Session {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<Map<String, Value>> for Session {
    fn from(m: Map<String, Value>) -> Self {
        Session {
            state: Arc::new(RwLock::new(m)),
        }
    }
}

impl From<Arc<RwLock<Map<String, Value>>>> for Session {
    fn from(m: Arc<RwLock<Map<String, Value>>>) -> Self {
        Session {
            state: Arc::clone(&m),
        }
    }
}

impl Sessionable for Session {
    fn state(&self) -> Result<RwLockReadGuard<'_, Map<String, Value>>, Error> {
        self.state
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    fn state_mut(&self) -> Result<RwLockWriteGuard<'_, Map<String, Value>>, Error> {
        self.state
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(if let Some(val) = self.state()?.get(key).cloned() {
            from_value(val)?
        } else {
            None
        })
    }

    fn set<T: DeserializeOwned + Serialize>(&self, key: &str, val: T) -> Result<Option<T>, Error> {
        Ok(
            if let Some(prev) = self.state_mut()?.insert(key.to_owned(), to_value(val)?) {
                from_value(prev)?
            } else {
                None
            },
        )
    }

    fn remove<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(if let Some(val) = self.state_mut()?.remove(key) {
            from_value(val)?
        } else {
            None
        })
    }

    fn clear(&self) -> Result<(), Error> {
        Ok(self.state_mut()?.clear())
    }
}
