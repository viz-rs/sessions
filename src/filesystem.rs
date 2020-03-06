//! FilesystemStore
//!
//! Stores the session in the filesystem store.

use std::{fmt, future::Future, io::Error, path::PathBuf, pin::Pin, sync::Arc};
use serde_json::{from_slice, to_vec};
use tokio::fs;

use crate::{Session, State, Storable};

#[derive(Clone, Debug)]
pub struct FilesystemStore {
    path: PathBuf,
}

impl FilesystemStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    async fn put(&self, sid: String, state: State) -> Result<(), Error> {
        fs::write(self.path.join(&sid), to_vec(&state)?).await
    }
}

impl Storable for FilesystemStore {
    fn get(&self, sid: &str) -> Pin<Box<dyn Future<Output = Result<Session, Error>> + Send + '_>> {
        let sid = sid.to_owned();
        Box::pin(async move {
            let file = fs::read(self.path.join(&sid)).await;
            let fresh = file.is_err();
            let session = Session::new(&sid, fresh, Arc::new(self.clone()));

            if fresh == false {
                let data = file?;
                // Should be a map `{}`
                if data.len() > 1 {
                    *session.state_mut().unwrap() = from_slice(&data)?;
                }
            }

            Ok(session)
        })
    }

    fn remove(&self, sid: &str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let sid = sid.to_owned();
        Box::pin(async move { fs::remove_file(self.path.join(&sid)).await })
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
        fmt::Debug::fmt(&self.path, f)
    }
}
