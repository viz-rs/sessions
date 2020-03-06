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

    async fn put(&self, id: String, state: State) -> Result<(), Error> {
        self.inner
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
            .insert(id, state);
        Ok(())
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
        Box::pin(async move { self.put(id, state).await })
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
