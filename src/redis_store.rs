use std::sync::Arc;

use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client, RedisResult};
use serde_json::{from_slice, to_vec};

use crate::{Session, SessionBeer, SessionStatus, Storable};

/// RedisStore
///
/// Stores the session to the redis.
#[derive(Clone, Debug)]
pub struct RedisStore {
    prefix: String,
    max_age: usize,
    client: Arc<Client>,
}

impl Default for RedisStore {
    fn default() -> Self {
        Self {
            prefix: "session:id".to_owned(),
            max_age: 60 * 60 * 24 * 7 * 2, // Two weeks
            client: Arc::new(Client::open("redis://127.0.0.1/").unwrap()),
        }
    }
}

impl RedisStore {
    /// Creates new Redis Store.
    #[inline]
    pub fn new(client: Arc<Client>, prefix: &str, max_age: usize) -> Self {
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
    pub async fn store(&self) -> RedisResult<Connection> {
        self.client.get_async_connection().await
    }
}

#[async_trait]
impl Storable for RedisStore {
    async fn get(&self, sid: &str) -> Session {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await {
            return session;
        }

        if let Ok(mut store) = self.store().await {
            if store.exists(self.prefix() + sid).await.unwrap_or_default() {
                if let Ok(raw) = store.get::<String, Vec<u8>>(self.prefix() + sid).await {
                    if let Ok(data) = from_slice(&raw) {
                        let SessionBeer { id, state, status } = &mut *session.beer().await;
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
        if let Ok(mut store) = self.store().await {
            store
                .del::<String, bool>(self.prefix() + sid)
                .await
                .unwrap_or_default()
        } else {
            false
        }
    }

    async fn save(&self, session: &Session) -> bool {
        if let Ok(mut store) = self.store().await {
            if let Ok(data) = to_vec(&session.state().await) {
                let id = session.id().await;
                let max_age = self.max_age();

                if max_age > 0 {
                    store.set_ex::<String, Vec<u8>, bool>(self.prefix() + &id, data, max_age)
                } else {
                    store.set::<String, Vec<u8>, bool>(self.prefix() + &id, data)
                }
                .await
                .unwrap_or_default()
            } else {
                false
            }
        } else {
            false
        }
    }
}
