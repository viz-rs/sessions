use std::{fmt, ops::Deref};

/// Sessions Config
pub struct Store<S, G, V> {
    /// Current Storage
    pub storage: S,
    /// Generates session id
    pub generate: G,
    /// Verifes session id
    pub verify: V,
}

impl<S, G, V> Store<S, G, V> {
    pub fn new(storage: S, generate: G, verify: V) -> Self {
        Self {
            storage,
            generate,
            verify,
        }
    }

    /// Gets current storage
    pub fn storage(&self) -> &S {
        &self.storage
    }
}

impl<S, G, V> AsRef<S> for Store<S, G, V> {
    fn as_ref(&self) -> &S {
        &self.storage
    }
}

impl<S, G, V> Deref for Store<S, G, V> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.storage
    }
}

impl<S, G, V> fmt::Debug for Store<S, G, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store").finish()
    }
}
