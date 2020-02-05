use std::{fmt, future::Future, io::Error, pin::Pin};

use crate::State;

pub trait Storable: Send + Sync + 'static {
    fn save(
        &self,
        name: String,
        state: State,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for dyn Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}
