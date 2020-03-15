//! Session
//!
//! Stores the values.

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};
use std::sync::Arc;

use crate::{RwLock, RwLockReadGuard, RwLockWriteGuard, State, Storable};

/// Session stores the values.
#[derive(Debug)]
pub struct Session {
    /// References the store.
    store: Arc<dyn Storable>,
    /// A session beer.
    /// Why not use `Rc<RefCell<Map<String, Value>>>`?
    /// See: https://github.com/hyperium/http/blob/master/src/extensions.rs
    beer: Arc<RwLock<SessionBeer>>,
}

impl Session {
    /// Creates new session.
    #[inline]
    pub fn new(store: Arc<dyn Storable>) -> Self {
        Self {
            store,
            beer: Arc::default(),
        }
    }

    /// References the store.
    pub fn store(&self) -> Arc<dyn Storable> {
        self.store.clone()
    }

    /// Reads the session beer.
    pub async fn beer(&self) -> RwLockReadGuard<'_, SessionBeer> {
        self.beer.read().await
    }

    /// Writes the session beer.
    pub async fn beer_mut(&self) -> RwLockWriteGuard<'_, SessionBeer> {
        self.beer.write().await
    }

    /// Gets the session id
    pub async fn id(&self) -> String {
        self.beer().await.id.clone()
    }

    /// Overrides a new id on this session
    pub async fn set_id(&self, id: String) {
        self.beer_mut().await.id = id;
    }

    /// Gets the session status
    pub async fn status(&self) -> SessionStatus {
        self.beer().await.status.clone()
    }

    /// Overrides a new status on this session
    pub async fn set_status(&self, status: SessionStatus) {
        self.beer_mut().await.status = status;
    }

    /// Gets the session state
    pub async fn state(&self) -> State {
        self.beer().await.state.clone()
    }

    /// Overrides a new state on this session
    pub async fn set_state(&self, state: State) {
        self.beer_mut().await.state = state;
    }

    /// Gets a reference to the value corresponding to the key.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(val) = self.beer().await.state.get(key).cloned() {
            from_value(val).ok()
        } else {
            None
        }
    }

    /// Sets a key-value pair into the state.
    ///
    /// If the state did not have this key present, `None` is returned.
    ///
    /// If the state did have this key present, the value is updated, and the old
    /// value is returned.
    pub async fn set<T: DeserializeOwned + Serialize>(&self, key: &str, val: T) -> Option<T> {
        // let SessionBeer { state, status: _ } = &mut *self.beer_mut()?;
        if let Some(prev) = self
            .beer_mut()
            .await
            .state
            .insert(key.to_owned(), to_value(val).ok()?)
        {
            // if *status != SessionStatus::Changed {
            //     *status = SessionStatus::Changed;
            // }
            from_value(prev).ok()
        } else {
            None
        }
    }

    /// Removes a key from the state, returning the value at the key if the key
    /// was previously in the state.
    pub async fn remove<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        // let SessionBeer { state, status: _ } = &mut *self.beer_mut()?;
        if let Some(val) = self.beer_mut().await.state.remove(key) {
            // if *status != SessionStatus::Changed {
            //     *status = SessionStatus::Changed;
            // }
            from_value(val).ok()
        } else {
            None
        }
    }

    /// Clears the state of this session.
    pub async fn clear(&self) {
        self.beer_mut().await.state.clear();
    }

    /// Saves this session to the store.
    pub async fn save(&self) -> bool {
        let id = self.id().await;
        if id.is_empty() {
            // Generates a new id.
            self.set_id(self.store.generate_sid().await).await;
        }
        self.store.save(self).await
    }

    /// Destroys this session from store.
    ///
    /// After changes session status to [`SessionStatus::Destroyed`].
    /// So we can check the session status when want to clean client cookie.
    pub async fn destroy(&self) -> bool {
        self.clear().await;
        self.set_status(SessionStatus::Destroyed).await;
        let id = self.id().await;
        if id.is_empty() {
            return true;
        }
        self.store.remove(&id).await
    }
}

/// Session Status
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    /// Session is created.
    Created,
    /// Session is existed.
    Existed,
    /// Session is changed.
    Changed,
    /// Session is destroyed.
    Destroyed,
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Created
    }
}

/// A Session Beer
#[derive(Debug, Clone, Default)]
pub struct SessionBeer {
    /// The session ID, and it shoulds be an unique ID.
    pub id: String,
    /// Stores the values.
    pub state: State,
    /// Stores the status.
    pub status: SessionStatus,
}
