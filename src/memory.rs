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

    async fn put(&self, sid: String, state: State) -> Result<(), Error> {
        self.inner
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
            .insert(sid, state);
        Ok(())
    }
}

impl Storable for MemoryStore {
    fn get(&self, sid: &str) -> Result<Session, Error> {
        let store = self
            .inner
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))?;

        let data = store.get(sid);
        let fresh = data.is_none();
        let session = Session::new(sid, fresh, Arc::new(self.clone()));

        if fresh == false {
            *session.state_mut().unwrap() = data.cloned().unwrap();
        }

        Ok(session)
    }

    fn remove(&self, sid: &str) -> Result<(), Error> {
        self.inner
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
            .remove(sid);
        Ok(())
    }

    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let sid = session.sid();
        let state = session.state().unwrap().clone();
        Box::pin(async move { self.put(sid, state).await })
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
