use serde::{de::DeserializeOwned, Serialize};
use std::io::Error;

use crate::Storable;

pub trait Sessionable<Store> {
    fn save();
    fn name(&self) -> String;
    fn store(self) -> Box<Store>;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error>;
    fn remove();
    fn clear();
}

#[derive(Debug)]
pub struct Session<Store> {
    name: String,
    store: Box<Store>,
}

impl<Store> Session<Store>
where
    Store: Storable,
{
    pub fn new(name: impl AsRef<str>, store: Store) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            store: Box::new(store),
        }
    }
}

impl<Store> Sessionable<Store> for Session<Store>
where
    Store: Storable,
{
    fn save() {}

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn store(self) -> Box<Store> {
        self.store
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        self.store.get(key)
    }
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
        self.store.set(key, value)
    }
    fn remove() {}
    fn clear() {}
}
