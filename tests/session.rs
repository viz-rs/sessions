use serde;
use serde::{de::DeserializeOwned, Serialize};
use sessions::{Session, Sessionable, Storable};
use std::collections::HashMap;
use std::io::Error;

#[test]
fn session() {
    struct MyStore {
        values: HashMap<String, String>,
    }

    impl MyStore {
        fn new() -> Self {
            Self {
                values: HashMap::new(),
            }
        }
    }

    impl Storable for MyStore {
        fn create(self, name: impl AsRef<str>) -> Session<Self> {
            Session::new(name, self)
        }

        fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<Option<T>, Error> {
            if let Some(value) = self.values.get(key.as_ref()) {
                Ok(Some(serde_json::from_str(value)?))
            } else {
                Ok(None)
            }
        }

        fn set<T: Serialize>(&mut self, key: impl AsRef<str>, value: T) -> Result<(), Error> {
            self.values
                .insert(key.as_ref().to_owned(), serde_json::to_string(&value)?);

            Ok(())
        }
    }

    // let session = Session::new("star-trek", MyStore {});
    let store = MyStore::new();
    let mut session = store.create("star-trek");

    assert_eq!(session.name(), "star-trek");

    assert_eq!(session.set("counter", 144).unwrap(), ());
    assert_eq!(session.get("counter").unwrap(), Some(144));
}
