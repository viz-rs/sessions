use serde_json::{from_slice, to_vec};
use std::{fmt, future::Future, io::Error, path::PathBuf, pin::Pin, sync::Arc};

#[cfg(feature = "async-std")]
use async_std::fs;
#[cfg(feature = "tokio")]
use tokio::fs;

use crate::{Session, SessionBeer, SessionStatus, Storable};

/// FilesystemStore
///
/// Stores the session in the filesystem store.
#[derive(Clone, Debug)]
pub struct FilesystemStore {
    path: PathBuf,
}

impl FilesystemStore {
    /// Creates new Filesystem Store
    #[inline]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Storable for FilesystemStore {
    fn get(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<Session, Error>> + Send + '_>> {
        let id = id.to_owned();
        Box::pin(async move {
            let file = fs::read(self.path.join(&id)).await;
            let session = Session::new(&id, Arc::new(self.clone()));

            {
                let SessionBeer { state, status } = &mut *session.beer_mut()?;
                if file.is_ok() {
                    *status = SessionStatus::Existed;
                    let data = file?;
                    // Should be a map `{}`
                    if data.len() > 1 {
                        *state = from_slice(&data)?;
                    }
                }
            }

            Ok(session)
        })
    }

    fn remove(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let id = id.to_owned();
        Box::pin(async move { fs::remove_file(self.path.join(&id)).await })
    }

    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
        let id = session.id();
        let state = session.state().unwrap().clone();
        Box::pin(async move { fs::write(self.path.join(&id), to_vec(&state)?).await })
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.path, f)
    }
}
