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
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Session, SessionBeer, SessionStatus, State, Storable};

type Map = HashMap<String, State>;

#[derive(Clone, Debug)]
pub struct MemoryStore {
    inner: Arc<RwLock<Map>>,
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
            let old_state = self.store()?.get(&id).cloned();
            let session = Session::new(&id, Arc::new(self.clone()));

            {
                let SessionBeer { state, status } = &mut *session.beer_mut()?;
                if old_state.is_some() {
                    *state = old_state.unwrap();
                    *status = SessionStatus::Existed;
                }
            }

            Ok(session)
        })
    }

    fn remove(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let id = id.to_owned();
        Box::pin(async move {
            self.store_mut()?.remove(&id);
            Ok(())
        })
    }

    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let id = session.id();
        let state = session.state().unwrap();
        Box::pin(async move {
            self.store_mut()?.insert(id, state);
            Ok(())
        })
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
