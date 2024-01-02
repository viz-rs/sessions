//! Sessions Core

#![forbid(unsafe_code)]

/// Set if the session was unchanged or inited.
pub const UNCHANGED: u8 = 0;

/// Set if the session has been destroied.
pub const PURGED: u8 = 1;

/// Set if the session has been renewed.
pub const RENEWED: u8 = 2;

/// Set if the session has been changed.
pub const CHANGED: u8 = 3;

/// A data state
pub type Data = std::collections::BTreeMap<String, serde_json::Value>;

mod state;
pub use state::State;

mod storage;
pub use storage::Storage;

mod store;
pub use store::Store;

pub use serde_json::Value;

#[cfg(feature = "session")]
mod session;
#[cfg(feature = "session")]
pub use session::Session;
