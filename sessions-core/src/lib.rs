//! Sessions Core

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, rustdoc::missing_doc_code_examples, unreachable_pub)]

mod config;
mod cookie_options;
mod session;
mod storage;

pub use anyhow::{anyhow, Error, Result};
pub use async_trait::async_trait;
pub use config::Config;
pub use cookie_options::CookieOptions;
pub use session::Session;
pub use storage::Storage;

/// A data state
pub type Data = data::Map<String, data::Value>;

#[allow(missing_docs)]
pub mod data {
    pub use ::serde::{de::DeserializeOwned, Serialize};
    pub use ::serde_json::{from_value, to_value, Map, Value};
}
