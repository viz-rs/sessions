//! Sessions Core

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, rustdoc::missing_doc_code_examples, unreachable_pub)]

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

mod config;
mod error;
mod session;
mod storage;

pub use async_trait::async_trait;
pub use config::Config;
pub use error::Error;
pub use serde_json::Value;
pub use session::Session;
pub use storage::Storage;
