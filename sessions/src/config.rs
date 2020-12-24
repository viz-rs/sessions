use std::sync::Arc;

use crate::CookieOptions;
use crate::Storage;

#[derive(Debug)]
pub struct Config<S: Storage> {
    cookie: CookieOptions,
    storage: Arc<S>,
}
