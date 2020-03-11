use async_trait::async_trait;
use std::{
    collections::HashMap,
    error::Error as ErrorExt,
    fmt,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Session, SessionBeer, SessionStatus, State, Storable};

type Map = HashMap<String, State>;

/// MemoryStore
///
/// Stores the session in an in-memory store.
#[derive(Clone, Debug)]
pub struct MemoryStore {
    inner: Arc<RwLock<Map>>,
}

impl MemoryStore {
    /// Creates new Memory Store
    #[inline]
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

    // async fn gen_sid(&self) -> Result<String, Error> {
    //     Ok("231".to_owned())
    // }
}

#[async_trait]
impl Storable for MemoryStore {
    async fn get(&self, sid: &str) -> Result<Session, Error> {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await? {
            return Ok(session);
        }

        let exists = self.store()?.contains_key(sid);

        if exists {
            let SessionBeer { id, state, status } = &mut *session.beer_mut()?;
            *state = self.store()?.get(sid).cloned().unwrap();
            *status = SessionStatus::Existed;
            *id = sid.to_owned();
        }

        Ok(session)
    }

    async fn remove(&self, sid: &str) -> Result<(), Error> {
        self.store_mut()?.remove(sid);
        Ok(())
    }

    async fn save(&self, session: &Session) -> Result<(), Error> {
        self.store_mut()?.insert(session.id()?, session.state()?);
        Ok(())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
