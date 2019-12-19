use serde::{de::DeserializeOwned, Serialize};
use std::io::Error;

use crate::Storable;

pub trait Sessionable<S> {
    fn save();
    fn name(&self) -> String;
    fn store(self) -> Box<S>;

    fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<Option<T>, Error>;
    fn set<T: Serialize>(&mut self, key: impl AsRef<str>, value: T) -> Result<(), Error>;
    fn remove();
    fn clear();
}

pub struct Session<S> {
    name: String,
    store: Box<S>,
}

impl<S> Session<S>
where
    S: Storable,
{
    pub fn new(name: impl AsRef<str>, store: S) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            store: Box::new(store),
        }
    }
}

impl<S> Sessionable<S> for Session<S>
where
    S: Storable,
{
    fn save() {}

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn store(self) -> Box<S> {
        self.store
    }

    fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<Option<T>, Error> {
        self.store.get(key)
    }
    fn set<T: Serialize>(&mut self, key: impl AsRef<str>, value: T) -> Result<(), Error> {
        self.store.set(key, value)
    }
    fn remove() {}
    fn clear() {}
}
