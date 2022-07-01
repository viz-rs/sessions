use thiserror::Error as ThisError;

/// Session Error
#[derive(ThisError, Debug)]
pub enum Error {
    /// RwLock Error
    #[error("{0}")]
    RwLock(String),
    /// Json Error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Connection Error
    #[error("{0}")]
    Connection(String),
}
