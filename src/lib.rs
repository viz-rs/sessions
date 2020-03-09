#![doc(html_root_url = "https://docs.rs/sessions/0.1.0")]
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
