//! Storable Trait
//!
//! A trait for session store.

use std::{fmt, future::Future, io::Error, pin::Pin};

use crate::Session;

pub trait Storable: Send + Sync + 'static {
    fn get(&self, sid: &str) -> Pin<Box<dyn Future<Output = Result<Session, Error>> + Send + '_>>;

    fn remove(&self, sid: &str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;

    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;

    /// @TODO: encode & decode the state

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for dyn Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}
