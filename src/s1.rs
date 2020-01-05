use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Map, Value};
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::Storable;

pub trait Sessionable<Store> {
    fn save(&mut self);
    fn state(&self) -> &Map<String, Value>;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    fn set<T: DeserializeOwned + Serialize>(&mut self, key: &str, val: T) -> Option<T>;
    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Option<T>;
    fn clear(&mut self);
}

#[derive(Debug, Default)]
pub struct Session<Store> {
    name: String,
    store: Arc<Store>,
    state: Map<String, Value>,
    is_new: bool,
}

impl<Store> Session<Store>
where
    Store: Storable,
{
    pub fn new(name: &str, store: Arc<Store>) -> Self {
        Self {
            name: name.to_owned(),
            state: Map::new(),
            is_new: true,
            store,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }
}

impl<Store> Sessionable<Store> for Session<Store>
where
    Store: Storable,
{
    fn save(&mut self) {
        // self.store.save(self);
    }

    fn state(&self) -> &Map<String, Value> {
        &self.state
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        // @TODO: logger error
        self.state
            .get(key)
            .and_then(|val| from_value(val.clone()).ok())
    }

    fn set<T: DeserializeOwned + Serialize>(&mut self, key: &str, val: T) -> Option<T> {
        // @TODO: logger error
        from_value(self.state.insert(key.to_owned(), to_value(val).ok()?)?).ok()
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Option<T> {
        // @TODO: logger error
        from_value(self.state.remove(key)?).ok()
    }

    fn clear(&mut self) {
        self.state.clear()
    }
}
