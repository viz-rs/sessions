use std::fmt;

use async_trait::async_trait;

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

    /// Generates a sid/UID fro a session by nanoid.
    async fn generate_sid(&self) -> String {
        nanoid::nanoid!(32)
    }

    /// Verifies a sid/UID.
    async fn verify_sid(&self, sid: &str) -> bool {
        sid.len() == 32
            && sid
                .chars()
                .all(|x| x.is_ascii_alphanumeric() || x == '_' || x == '-')
    }

    /// Set the Storable's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl fmt::Debug for dyn Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store").field("type", &self.name()).finish()
    }
}
