//! MemoryStore

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

    async fn save_data(&self, name: String, state: State) -> Result<(), Error> {
        self.inner
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
            .insert(name, state);
        Ok(())
    }
}

impl Storable for MemoryStore {
    fn get(&self, name: &str) -> Result<Session, Error> {
        let store = self
            .inner
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))?;

        Ok(if let Some(data) = store.get(name).cloned() {
            let session = self.touch(name, false);
            *session.state_mut().unwrap() = data;
            session
        } else {
            self.touch(name, true)
        })
    }

    fn touch(&self, name: &str, fresh: bool) -> Session {
        Session::new(name, fresh, Arc::new(self.clone()))
    }

    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let name = session.name();
        let state = session.state().unwrap().clone();
        Box::pin(async move { self.save_data(name, state).await })
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
