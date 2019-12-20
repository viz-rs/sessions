use crate::Session;
use serde::{de::DeserializeOwned, Serialize};
use std::io::Error;

pub trait Storable: Send + Sync + Sized + 'static {
    fn create(self, name: &str) -> Session<Self>;
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error>;
    // fn save();
}
