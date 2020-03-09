//! Sessions
//!
//! Provides memory and filesystem sessions and infrastructure
//! for custom session backends.
//!

#![deny(unsafe_code)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use memory::MemoryStore;

#[cfg(feature = "filesystem")]
mod filesystem;
#[cfg(feature = "filesystem")]
pub use filesystem::FilesystemStore;

// #[cfg(feature = "cookie")]
// mod cookie;

// #[cfg(feature = "mongodb")]
// mod mongodb;
// #[cfg(feature = "redis")]
// mod redis;

mod options;
pub use options::Options;

mod session;
pub use session::{Session, SessionBeer, SessionStatus};

mod state;
pub use state::State;

mod store;
pub use store::Storable;
