use futures::future::BoxFuture;
use std::fmt;
use std::io::Error;

use crate::State;

pub trait Storable: Send + Sync + 'static {
    fn save(&self, name: String, state: State) -> BoxFuture<'_, Result<(), Error>>;
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for dyn Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}
