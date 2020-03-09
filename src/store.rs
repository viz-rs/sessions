use std::{fmt, future::Future, io::Error, pin::Pin};

use crate::Session;

/// Storable Trait
///
/// A trait for session store.
pub trait Storable: Send + Sync + 'static {
    /// Gets a session by the id.
    fn get(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<Session, Error>> + Send + '_>>;

    /// Removes a session by the id.
    fn remove(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;

    /// Saves a session.
    fn save(
        &self,
        session: &Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;

    /// @TODO: encode & decode the state

    /// Just hacks for debuging the Store.
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for dyn Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}
