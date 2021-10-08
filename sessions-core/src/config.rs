use std::{fmt, ops::Deref, time::Duration};

use crate::{CookieOptions, Storage};

/// Sessions Config
pub struct Config<S: Storage> {
    /// Cookie Options
    pub cookie: CookieOptions,
    /// Current Storage
    pub storage: S,
    /// Generates session id
    pub generate: Box<dyn Send + Sync + 'static + Fn() -> String>,
    /// Verifes session id
    pub verify: Box<dyn Send + Sync + 'static + Fn(&str) -> bool>,
}

impl<S: Storage> Config<S> {
    /// Gets current storage
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Gets cookie options
    pub fn cookie(&self) -> &CookieOptions {
        &self.cookie
    }

    /// Gets cookie's max_age or session's expries
    pub fn max_age(&self) -> Duration {
        self.cookie.max_age
    }

    /// Generates a session id
    pub fn generate(&self) -> String {
        (self.generate)()
    }

    /// Verifes a session id
    pub fn verify(&self, key: &str) -> bool {
        (self.verify)(key)
    }
}

impl<S: Storage> fmt::Debug for Config<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("cookie", &self.cookie)
            .field("storage", &self.storage)
            .finish()
    }
}

impl<S: Storage> AsRef<S> for Config<S> {
    fn as_ref(&self) -> &S {
        &self.storage
    }
}

impl<S: Storage> Deref for Config<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.storage
    }
}
