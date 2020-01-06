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
#[cfg(feature = "redis")]
mod redis;

pub mod session;

// pub use store::Storable;
