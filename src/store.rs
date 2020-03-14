use async_trait::async_trait;
use std::fmt;

use crate::Session;

/// Storable Trait
///
/// A trait for session store.
#[async_trait]
pub trait Storable: Send + Sync + 'static {
    /// Gets a session by the sid.
    /// Or returns a new session when not found.
    async fn get(&self, sid: &str) -> Session;

    /// Removes a session by the sid.
    async fn remove(&self, sid: &str) -> bool;

    /// Saves a session.
    async fn save(&self, session: &Session) -> bool;

    #[cfg(not(feature = "nanoid"))]
    /// Generates a sid/UID fro a session.
    async fn generate_sid(&self) -> String;
    #[cfg(feature = "nanoid")]
    /// Generates a sid/UID fro a session by nanoid.
    async fn generate_sid(&self) -> String {
        nanoid::nanoid!(32)
    }

    #[cfg(not(feature = "nanoid"))]
    /// Verifies a sid/UID.
    async fn verify_sid(&self, sid: &str) -> bool {
        sid.len() > 0
    }
    #[cfg(feature = "nanoid")]
    /// Verifies a sid/UID.
    async fn verify_sid(&self, sid: &str) -> bool {
        sid.len() == 32
    }

    /// @TODO: encode & decode the state

    /// Just hacks for debuging the Store.
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for dyn Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}
