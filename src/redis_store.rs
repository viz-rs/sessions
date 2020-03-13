use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};
use serde_json::{from_slice, to_vec};

use std::{
    fmt,
    io::{Error, ErrorKind},
    sync::Arc,
};

use crate::{Session, SessionBeer, SessionStatus, Storable};

/// RedisStore
///
/// Stores the session to the redis.
#[derive(Clone, Debug)]
pub struct RedisStore {
    prefix: String,
    max_age: usize,
    client: Client,
}

impl RedisStore {
    /// Creates new Redis Store.
    #[inline]
    pub fn new(client: Client, prefix: &str, max_age: usize) -> Self {
        Self {
            client,
            max_age,
            prefix: prefix.to_owned(),
        }
    }

    /// Gets the prefix of key.
    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    /// Gets the ttl of key.
    pub fn max_age(&self) -> usize {
        self.max_age
    }

    /// Gets the redis connection.
    pub async fn store(&self) -> Result<Connection, Error> {
        self.client
            .get_async_connection()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
}

#[async_trait]
impl Storable for RedisStore {
    async fn get(&self, sid: &str) -> Session {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await {
            return session;
        }

        let store = self.store().await;

        if store.is_err() {
            return session;
        }

        if let Ok(mut store) = store {
            if store
                .exists(self.prefix() + sid)
                .await
                .unwrap_or_else(|_| false)
            {
                if let Ok(raw) = store.get::<String, Vec<u8>>(self.prefix() + sid).await {
                    if let Ok(data) = from_slice(&raw) {
                        let SessionBeer { id, state, status } = &mut *session.beer_mut().await;

                        *state = data;
                        *status = SessionStatus::Existed;
                        *id = sid.to_owned();
                    }
                }
            }
        }

        session
    }

    async fn remove(&self, sid: &str) -> bool {
        let store = self.store().await;

        if store.is_err() {
            return false;
        }

        store
            .unwrap()
            .del::<String, bool>(self.prefix() + sid)
            .await
            .unwrap_or_else(|_| false)
    }

    async fn save(&self, session: &Session) -> bool {
        let store = self.store().await;

        if store.is_err() {
            return false;
        }

        let mut store = store.unwrap();

        if let Ok(data) = to_vec(&session.state().await) {
            let max_age = self.max_age();
            let id = session.id().await;

            if max_age > 0 {
                store.set_ex::<String, Vec<u8>, bool>(self.prefix() + &id, data, max_age)
            } else {
                store.set::<String, Vec<u8>, bool>(self.prefix() + &id, data)
            }
            .await
            .unwrap_or_else(|_| false)
        } else {
            false
        }
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.client, f)
    }
}
