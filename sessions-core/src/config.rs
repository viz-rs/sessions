use std::{fmt, ops::Deref};

use crate::Storage;

/// Sessions Config
pub struct Config<S, G, V> {
    /// Current Storage
    pub storage: S,
    /// Generates session id
    pub generate: G,
    /// Verifes session id
    pub verify: V,
}

impl<S, G, V> Config<S, G, V>
where
    S: Storage,
    G: Fn() -> String,
    V: Fn(&str) -> bool,
{
    /// Gets current storage
    pub fn storage(&self) -> &S {
        &self.storage
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

impl<S, G, V> fmt::Debug for Config<S, G, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config").finish()
    }
}

impl<S, G, V> AsRef<S> for Config<S, G, V> {
    fn as_ref(&self) -> &S {
        &self.storage
    }
}

impl<S, G, V> Deref for Config<S, G, V> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.storage
    }
}
