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

use crate::{State, Storable};

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Created,
    Existed,
    Changed,
    Destroyed,
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Created
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionBeer {
    pub state: State,
    pub status: SessionStatus,
}

#[derive(Debug)]
pub struct Session {
    /// session ID, and it shoulds be an unique ID.
    id: String,
    /// Stores session
    store: Arc<dyn Storable>,
    /// Why not use `Rc<RefCell<Map<String, Value>>>`?
    /// See: https://github.com/hyperium/http/blob/master/src/extensions.rs
    beer: Arc<RwLock<SessionBeer>>,
}

impl Session {
    #[inline]
    pub fn new(id: &str, store: Arc<impl Storable>) -> Self {
        Self {
            store,
            id: id.to_owned(),
            beer: Arc::default(),
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn store(&self) -> Arc<dyn Storable> {
        self.store.clone()
    }

    pub fn status(&self) -> Result<SessionStatus, Error> {
        Ok(self.beer()?.status.clone())
    }

    pub fn set_status(&self, status: SessionStatus) -> Result<(), Error> {
        self.beer_mut()?.status = status;
        Ok(())
    }

    pub fn state(&self) -> Result<State, Error> {
        Ok(self.beer()?.state.clone())
    }

    pub fn set_state(&self, state: State) -> Result<(), Error> {
        self.beer_mut()?.state = state;
        Ok(())
    }

    pub fn beer(&self) -> Result<RwLockReadGuard<'_, SessionBeer>, Error> {
        self.beer
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    pub fn beer_mut(&self) -> Result<RwLockWriteGuard<'_, SessionBeer>, Error> {
        self.beer
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(if let Some(val) = self.beer()?.state.get(key).cloned() {
            from_value(val)?
        } else {
            None
        })
    }

    pub fn set<T: DeserializeOwned + Serialize>(
        &self,
        key: &str,
        val: T,
    ) -> Result<Option<T>, Error> {
        let SessionBeer { state, status: _ } = &mut *self.beer_mut()?;
        Ok(
            if let Some(prev) = state.insert(key.to_owned(), to_value(val)?) {
                // if *status != SessionStatus::Changed {
                //     *status = SessionStatus::Changed;
                // }
                from_value(prev)?
            } else {
                None
            },
        )
    }

    pub fn remove<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        let SessionBeer { state, status: _ } = &mut *self.beer_mut()?;
        Ok(if let Some(val) = state.remove(key) {
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
        self.store.save(self).await
    }

    /// Destroys this session.
    pub async fn destroy(&self) -> Result<(), Error> {
        self.store.remove(&self.id).await?;
        self.set_status(SessionStatus::Destroyed)?;
        Ok(())
    }
}
