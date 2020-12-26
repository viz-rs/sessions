use std::sync::Arc;

use crate::CookieOptions;
use crate::Storage;

#[derive(Debug)]
pub struct Config<S, G, V> {
    cookie: CookieOptions,
    storage: Arc<S>,
    pub generate: G,
    pub verify: V,
}

impl<S, G, V> Config<S, G, V>
where
    S: Storage,
    G: Fn() -> String,
    V: Fn(&str) -> bool,
{
    pub fn storage(&self) -> Arc<S> {
        self.storage.clone()
    }

    pub fn cookie(&self) -> &CookieOptions {
        &self.cookie
    }
}
