use serde::{de::DeserializeOwned, Serialize};

use std::io::Error;

use crate::Sessionable;

pub trait Storable: Send + Sync + Sized + 'static {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error>;
    // fn save<S: Sessionable<Self>>(&mut self, session: S);
}
