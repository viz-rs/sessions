//! State Map
//!
//! Stores the value by the key.
//! Based on [`serde_json::Map`].

use serde_json::{Map, Value};

pub type State = Map<String, Value>;
