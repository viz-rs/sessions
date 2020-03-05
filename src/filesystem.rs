//! MemoryStore

use std::{
    collections::HashMap,
    error::Error as ErrorExt,
    fmt,
    future::Future,
    io::{Error, ErrorKind},
    path::Path,
    pin::Pin,
    sync::{Arc, RwLock},
};

use crate::{Session, State, Storable};

#[derive(Clone, Debug)]
pub struct FilesystemStore {
    path: Path,
}

impl FilesystemStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::default(),
        }
    }

    // async fn save_data(&self, key: String, state: State) -> Result<(), Error> {
    //     self.inner
    //         .write()
    //         .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
    //         .insert(key, state);
    //     Ok(())
    // }
}

// impl Storable for MemoryStore {
//     fn create(&self, key: &str) -> Session {
//         Session::new(key, Arc::new(self.clone()))
//     }

//     fn save(
//         &self,
//         session: &Session,
//     ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>> {
//         let key = session.key();
//         let state = session.state().unwrap().clone();
//         Box::pin(async move { self.save_data(key, state).await })
//     }

//     fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::Debug::fmt(&self.inner, f)
//     }
// }
