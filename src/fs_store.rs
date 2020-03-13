use async_trait::async_trait;
use serde_json::{from_slice, to_vec};
use std::{fmt, path::PathBuf, sync::Arc};

#[cfg(all(not(feature = "tokio"), feature = "async-std"))]
use async_std::fs;
#[cfg(all(feature = "tokio", not(feature = "async-std")))]
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
    async fn get(&self, sid: &str) -> Session {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await {
            return session;
        }

        let file = fs::read(self.path.join(sid)).await;

        if file.is_ok() {
            let raw = file.unwrap();

            // Should be a map `{}`
            if raw.len() < 2 {
                return session;
            }

            let SessionBeer { id, state, status } = &mut *session.beer_mut().await;

            if let Ok(data) = from_slice(&raw) {
                *state = data;
                *status = SessionStatus::Existed;
                *id = sid.to_owned();
            }
        }

        session
    }

    async fn remove(&self, sid: &str) -> bool {
        fs::remove_file(self.path.join(sid)).await.is_ok()
    }

    async fn save(&self, session: &Session) -> bool {
        if let Ok(data) = to_vec(&session.state().await) {
            fs::write(self.path.join(session.id().await), data)
                .await
                .is_ok()
        } else {
            false
        }
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.path, f)
    }
}
