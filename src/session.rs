//! Session
//!
//! Stores the values.

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};
use std::{
    error::Error as ErrorExt,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

// @TODO: async/await?
// #[cfg(feature = "async-std")]
// use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
// #[cfg(feature = "tokio")]
// use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{State, Storable};

/// Session stores the values.
///
/// # Examples
///
/// ```
/// let id = uuid();
/// let store = Arc::new(CustomStore::new());
/// let session = Session::new(&id, store.clone());
///
/// session.set("counter", 0);
/// session.get::<usize>("counter");
/// session.remove("counter");
///
/// session.save().await?;
/// session.destroy().await?;
/// ```
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
    pub fn new(store: Arc<impl Storable>) -> Self {
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
    pub fn beer(&self) -> Result<RwLockReadGuard<'_, SessionBeer>, Error> {
        self.beer
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    /// Writes the session beer.
    pub fn beer_mut(&self) -> Result<RwLockWriteGuard<'_, SessionBeer>, Error> {
        self.beer
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    /// Gets the session id
    pub fn id(&self) -> Result<String, Error> {
        Ok(self.beer()?.id.clone())
    }

    /// Overrides a new id on this session
    pub fn set_id(&self, id: String) -> Result<(), Error> {
        self.beer_mut()?.id = id;
        Ok(())
    }

    /// Gets the session status
    pub fn status(&self) -> Result<SessionStatus, Error> {
        Ok(self.beer()?.status.clone())
    }

    /// Overrides a new status on this session
    pub fn set_status(&self, status: SessionStatus) -> Result<(), Error> {
        self.beer_mut()?.status = status;
        Ok(())
    }

    /// Gets the session state
    pub fn state(&self) -> Result<State, Error> {
        Ok(self.beer()?.state.clone())
    }

    /// Overrides a new state on this session
    pub fn set_state(&self, state: State) -> Result<(), Error> {
        self.beer_mut()?.state = state;
        Ok(())
    }

    /// Gets a reference to the value corresponding to the key.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(if let Some(val) = self.beer()?.state.get(key).cloned() {
            from_value(val)?
        } else {
            None
        })
    }

    /// Sets a key-value pair into the state.
    ///
    /// If the state did not have this key present, `None` is returned.
    ///
    /// If the state did have this key present, the value is updated, and the old
    /// value is returned.
    pub fn set<T: DeserializeOwned + Serialize>(
        &self,
        key: &str,
        val: T,
    ) -> Result<Option<T>, Error> {
        // let SessionBeer { state, status: _ } = &mut *self.beer_mut()?;
        Ok(
            if let Some(prev) = self
                .beer_mut()?
                .state
                .insert(key.to_owned(), to_value(val)?)
            {
                // if *status != SessionStatus::Changed {
                //     *status = SessionStatus::Changed;
                // }
                from_value(prev)?
            } else {
                None
            },
        )
    }

    /// Removes a key from the state, returning the value at the key if the key
    /// was previously in the state.
    pub fn remove<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        // let SessionBeer { state, status: _ } = &mut *self.beer_mut()?;
        Ok(if let Some(val) = self.beer_mut()?.state.remove(key) {
            // if *status != SessionStatus::Changed {
            //     *status = SessionStatus::Changed;
            // }
            from_value(val)?
        } else {
            None
        })
    }

    /// Clears the state of this session.
    pub fn clear(&self) -> Result<(), Error> {
        Ok(self.beer_mut()?.state.clear())
    }

    /// Saves this session to the store.
    pub async fn save(&self) -> Result<(), Error> {
        let id = self.id()?;
        if id.is_empty() {
            // Generates a new id.
            self.set_id(self.store.gen_sid().await?)?;
        }
        self.store.save(self).await
    }

    /// Destroys this session from store.
    ///
    /// After changes session status to [`SessionStatus::Destroyed`].
    /// So we can check the session status when want to clean client cookie.
    pub async fn destroy(&self) -> Result<(), Error> {
        let id = self.id()?;
        if !id.is_empty() {
            self.store.remove(&id).await?;
        }
        self.clear()?;
        self.set_status(SessionStatus::Destroyed)?;
        Ok(())
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
