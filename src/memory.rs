//! MemoryStore
//!
//! Stores the session in an in-memory store.

use std::{
    collections::HashMap,
    error::Error as ErrorExt,
    fmt,
    future::Future,
    io::{Error, ErrorKind},
    pin::Pin,
    sync::{Arc, RwLock},
};

use crate::{Session, State, Storable};

#[derive(Clone, Debug)]
pub struct MemoryStore {
    inner: Arc<RwLock<HashMap<String, State>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::default(),
        }
    }

    fn store(&self) -> Result<RwLockReadGuard<'_, Map>, Error> {
        self.inner
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    fn store_mut(&self) -> Result<RwLockWriteGuard<'_, Map>, Error> {
        self.inner
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }
}

impl Storable for MemoryStore {
    fn get(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<Session, Error>> + Send + '_>> {
        let id = id.to_owned();
        Box::pin(async move {
            let store = self
                .inner
                .read()
                .map_err(|e| Error::new(ErrorKind::Other, e.description()))?;

            let state = store.get(&id);
            let fresh = state.is_none();
            let session = Session::new(&id, fresh, Arc::new(self.clone()));

            if fresh == false {
                *session.state_mut().unwrap() = state.cloned().unwrap();
            }

            Ok(session)
        })
    }

    fn remove(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let id = id.to_owned();
        Box::pin(async move {
            self.inner
                .write()
                .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
                .remove(&id);
            Ok(())
        })
    }

    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let id = session.id();
        let state = session.state().unwrap().clone();
        Box::pin(async move {
            self.store_mut()?.insert(id, state);
            Ok(())
        })
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
