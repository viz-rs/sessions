use async_trait::async_trait;
use std::{collections::HashMap, fmt, sync::Arc};

use crate::{
    RwLock, RwLockReadGuard, RwLockWriteGuard, Session, SessionBeer, SessionStatus, State, Storable,
};

/// MemoryStore
///
/// Stores the session in an in-memory store.
#[derive(Clone, Debug)]
pub struct MemoryStore {
    inner: Arc<RwLock<HashMap<String, State>>>,
}

impl MemoryStore {
    /// Creates new Memory Store
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Arc::default(),
        }
    }

    async fn store(&self) -> RwLockReadGuard<'_, HashMap<String, State>> {
        self.inner.read().await
    }

    async fn store_mut(&self) -> RwLockWriteGuard<'_, HashMap<String, State>> {
        self.inner.write().await
    }
}

#[async_trait]
impl Storable for MemoryStore {
    async fn get(&self, sid: &str) -> Session {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await {
            return session;
        }

        let store = self.store().await;

        if store.contains_key(sid) {
            let SessionBeer { id, state, status } = &mut *session.beer_mut().await;

            if let Some(data) = store.get(sid).cloned() {
                *state = data;
                *status = SessionStatus::Existed;
                *id = sid.to_owned();
            }
        }

        session
    }

    async fn remove(&self, sid: &str) -> bool {
        self.store_mut().await.remove(sid).is_some()
    }

    async fn save(&self, session: &Session) -> bool {
        self.store_mut()
            .await
            .insert(session.id().await, session.state().await)
            .map_or_else(|| true, |_| true)
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
