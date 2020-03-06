//! State Map
//!
//! Stores the value by the key.

use serde_json::{Map, Value};

pub type State = Map<String, Value>;
