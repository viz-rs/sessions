//! Sessions provides cookie and filesystem sessions and infrastructure for custom session backends.

#![deny(unsafe_code)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

#[cfg(feature = "cookie")]
mod cookie;
#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use memory::MemoryStore;
#[cfg(feature = "mongodb")]
mod mongodb;
#[cfg(feature = "redis")]
mod redis;

mod options;
mod session;
mod state;
mod store;

pub use options::Options;
pub use session::Session;
pub use state::State;
pub use store::Storable;
