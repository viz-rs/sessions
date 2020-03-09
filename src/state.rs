use serde_json::{Map, Value};

/// State Map
///
/// Stores the value by the key.
/// Based on [`serde_json::Map`](https://docs.rs/serde_json/latest/serde_json/map/index.html).
pub type State = Map<String, Value>;
