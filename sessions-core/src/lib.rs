//! Sessions

mod config;
mod cookie_options;
mod session;
mod storage;

pub use anyhow::{anyhow, Error, Result};
pub use async_trait::async_trait;
pub use config::{Config, GenerateFn, VerifyFn};
pub use cookie_options::CookieOptions;
pub use session::Session;
pub use storage::Storage;
pub type Data = data::Map<String, data::Value>;

pub mod data {
    pub use ::serde::{de::DeserializeOwned, Serialize};
    pub use ::serde_json::{from_value, to_value, Map, Value};
}
