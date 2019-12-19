use crate::Session;
use serde::{de::DeserializeOwned, Serialize};
use std::io::Error;

pub trait Storable: Send + Sync + Sized + 'static {
    fn create(self, name: impl AsRef<str>) -> Session<Self>;
    fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<Option<T>, Error>;
    fn set<T: Serialize>(&mut self, key: impl AsRef<str>, value: T) -> Result<(), Error>;
    // fn save();
}
