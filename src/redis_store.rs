use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};
use serde_json::{from_slice, to_vec};

use std::{
    error::Error as ErrorExt,
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
    /// Creates new Memory Store.
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
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }
}

#[async_trait]
impl Storable for RedisStore {
    async fn get(&self, sid: &str) -> Result<Session, Error> {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await? {
            return Ok(session);
        }

        let exists = self
            .store()
            .await?
            .exists(self.prefix() + sid)
            .await
            .unwrap_or_else(|_| false);

        if exists {
            let data = self
                .store()
                .await?
                .get::<String, Vec<u8>>(self.prefix() + sid)
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e.description()))?;

            let SessionBeer { id, state, status } = &mut *session.beer_mut()?;

            *state = from_slice(&data)?;
            *status = SessionStatus::Existed;
            *id = sid.to_owned();
        }

        Ok(session)
    }

    async fn remove(&self, sid: &str) -> Result<(), Error> {
        self.store()
            .await?
            .del::<String, ()>(self.prefix() + sid)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    async fn save(&self, session: &Session) -> Result<(), Error> {
        let max_age = self.max_age();
        let mut store = self.store().await?;
        let res = if max_age > 0 {
            store.set_ex::<String, Vec<u8>, ()>(
                self.prefix() + &session.id()?,
                to_vec(&session.state()?)?,
                max_age,
            )
        } else {
            store.set::<String, Vec<u8>, ()>(
                self.prefix() + &session.id()?,
                to_vec(&session.state()?)?,
            )
        };
        res.await
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.client, f)
    }
}
