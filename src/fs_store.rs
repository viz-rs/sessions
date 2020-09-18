use std::{fs, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use blocking::unblock;
use serde_json::{from_slice, to_vec};

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

        let path = self.path.join(sid);
        if let Ok(raw) = unblock(|| fs::read(path)).await {
            // Should be a map `{}`
            if raw.len() < 2 {
                return session;
            }

            if let Ok(data) = from_slice(&raw) {
                let SessionBeer { id, state, status } = &mut *session.write().await;
                *state = data;
                *status = SessionStatus::Existed;
                *id = sid.to_owned();
            }
        }

        session
    }

    async fn remove(&self, sid: &str) -> bool {
        let path = self.path.join(sid);
        unblock(|| fs::remove_file(path)).await.is_ok()
    }

    async fn save(&self, session: &Session) -> bool {
        if let Ok(data) = to_vec(&session.state().await) {
            let sid = self.path.join(session.id().await);
            unblock(|| fs::write(sid, data)).await.is_ok()
        } else {
            false
        }
    }
}
