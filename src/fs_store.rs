use async_trait::async_trait;
use serde_json::{from_slice, to_vec};
use std::{fmt, io::Error, path::PathBuf, sync::Arc};

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

#[async_trait]
impl Storable for FilesystemStore {
    async fn get(&self, sid: &str) -> Result<Session, Error> {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await? {
            return Ok(session);
        }

        let file = fs::read(self.path.join(sid)).await;
        let exists = file.is_ok();

        if exists {
            let SessionBeer { id, state, status } = &mut *session.beer_mut()?;
            let data = file?;
            // Should be a map `{}`
            if data.len() > 1 {
                *state = from_slice(&data)?;
            }
            *status = SessionStatus::Existed;
            *id = sid.to_owned();
        }

        Ok(session)
    }

    async fn remove(&self, sid: &str) -> Result<(), Error> {
        fs::remove_file(self.path.join(sid)).await
    }

    async fn save(&self, session: &Session) -> Result<(), Error> {
        fs::write(self.path.join(session.id()?), to_vec(&session.state()?)?).await
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.path, f)
    }
}
