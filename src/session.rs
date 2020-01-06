use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Map, Value};
use std::io::{Error, ErrorKind};

use crate::Storable;

pub trait Sessionable {
    // fn save(&mut self);

    fn state(&self) -> &Map<String, Value>;
    fn state_mut(&mut self) -> &mut Map<String, Value>;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    fn set<T: DeserializeOwned + Serialize>(&mut self, key: &str, val: T) -> Option<T>;
    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Option<T>;
    fn clear(&mut self);
}

#[derive(Debug, Clone, Default)]
pub struct Session {
    state: Map<String, Value>,
}

impl Session {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Sessionable for Session {
    // fn save(&mut self) {
    // self.store.save(self);
    // }

    fn state(&self) -> &Map<String, Value> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.state
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
