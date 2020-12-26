use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::Config;
use crate::Storage;

#[derive(Debug)]
pub struct Session<S, G, V> {
    pub id: String,
    pub fresh: AtomicBool,
    config: Arc<Config<S, G, V>>,
}

impl<S, G, V> Session<S, G, V>
where
    S: Storage,
    G: Fn() -> String,
    V: Fn(&str) -> bool,
{
    /// Gets the session id
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Gets the session status
    pub fn fresh(&self) -> bool {
        self.fresh.load(Ordering::Relaxed)
    }

    /// Gets a value by the key
    pub async fn get(&self, key: &str) {}

    /// Sets a value by the key
    pub async fn set<T>(&self, key: &str, value: T) {}

    /// Removes a value
    pub async fn remove(&self, key: &str) {}

    /// Clears the state
    pub async fn clear(&self) {}

    /// Saves the current state to the store
    pub async fn save(&self) {}

    /// Renews the new state
    pub async fn renew(&mut self) {
        self.id = (self.config.generate)();
    }

    /// Destroys the current state from store
    pub async fn destroy(&self) {}
}
