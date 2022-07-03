use std::sync::{atomic::AtomicU8, RwLock};

use crate::Data;

/// The Session State
#[derive(Debug, Default)]
pub struct State {
    /// status
    pub status: AtomicU8,
    /// raw data
    pub data: RwLock<Data>,
}
