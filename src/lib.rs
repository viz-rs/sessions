#![doc(html_root_url = "https://docs.rs/sessions/0.0.1")]
#![deny(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    nonstandard_style,
    future_incompatible
)]
#![deny(intra_doc_link_resolution_failure)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Sessions
//!
//! Provides memory and filesystem sessions and infrastructure
//! for custom session backends.
//!

#[cfg(all(not(feature = "tokio"), feature = "async-std"))]
pub use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(all(feature = "tokio", not(feature = "async-std")))]
pub use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "memory-store")]
mod memory_store;
#[cfg(feature = "memory-store")]
pub use memory_store::MemoryStore;

#[cfg(feature = "fs-store")]
mod fs_store;
#[cfg(feature = "fs-store")]
pub use fs_store::FilesystemStore;

#[cfg(feature = "redis-store")]
mod redis_store;
#[cfg(feature = "redis-store")]
pub use redis_store::RedisStore;

mod session;
pub use session::{Session, SessionBeer, SessionStatus};

mod state;
pub use state::State;

mod store;
pub use store::Storable;
