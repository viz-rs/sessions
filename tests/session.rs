use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};
use sessions::{Session, State, Storable};
use std::{
    collections::HashMap,
    error::Error as ErrorExt,
    fmt,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock},
};
use tokio::runtime::Runtime;

#[test]
fn session() {
    #[derive(Clone, Debug)]
    struct MyStore {
        values: Arc<RwLock<HashMap<String, String>>>,
    }

    impl MyStore {
        fn new() -> Self {
            Self {
                values: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        async fn save_data(&self, name: String, state: State) -> Result<(), Error> {
            self.values
                .write()
                .map_err(|e| Error::new(ErrorKind::Other, e.description()))?
                .insert(name, serde_json::to_string(&state)?);
            Ok(())
        }
    }

    impl Storable for MyStore {
        fn save(&self, name: String, state: State) -> BoxFuture<'_, Result<(), Error>> {
            Box::pin(async move { self.save_data(name, state).await })
        }

        fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.values, f)
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        age: u32,
        name: String,
    }

    let store = MyStore::new();

    let store = Arc::new(store);

    let rt = Runtime::new().unwrap();

    for i in 0..10 {
        let name = format!("trek-{}", i);
        let store = store.clone();

        rt.spawn(async move {
            println!(" ========> {} <=========", i);
            let session = Session::new(&name, store);

            assert_eq!(session.name(), name);

            assert_eq!(session.set("counter", 144).unwrap(), None);
            assert_eq!(session.set("number", 233).unwrap(), None);
            assert_eq!(session.get::<usize>("counter").unwrap(), Some(144));
            assert_eq!(session.get::<u32>("number").unwrap(), Some(233));
            assert_eq!(
                session
                    .set(
                        "user",
                        User {
                            age: 23,
                            name: "Jordan".to_owned(),
                        }
                    )
                    .unwrap(),
                None
            );
            assert_eq!(
                session
                    .set(
                        "user",
                        User {
                            age: 37,
                            name: "Kobe".to_owned(),
                        }
                    )
                    .unwrap(),
                Some(User {
                    age: 23,
                    name: "Jordan".to_owned(),
                })
            );
            let user: Option<User> = session.get::<User>("user").unwrap();
            assert_eq!(
                user,
                Some(User {
                    age: 37,
                    name: "Kobe".to_owned(),
                })
            );

            let mut state = Map::new();
            state.insert("counter".to_owned(), json!(144));
            state.insert("number".to_owned(), json!(233));
            state.insert(
                "user".to_owned(),
                json!(User {
                    age: 37,
                    name: "Kobe".to_owned(),
                }),
            );
            assert_eq!(session.state().unwrap().clone(), state);
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                r#"{"counter":144,"number":233,"user":{"age":37,"name":"Kobe"}}"#
            );
            assert_eq!(
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                r#"{"counter":144,"number":233,"user":{"age":37,"name":"Kobe"}}"#
            );

            assert_eq!(session.remove("number").unwrap(), Some(json!(233)));
            assert_eq!(session.remove::<f32>("counter").unwrap(), Some(144.0));
            assert_eq!(session.get::<u32>("counter").unwrap(), None);
            assert_eq!(session.remove::<usize>("counter").unwrap(), None);

            state.remove("number");
            state.remove("counter");
            assert_eq!(session.state().unwrap().clone(), state);

            assert_eq!(session.clear().unwrap(), ());
            assert_eq!(session.state().unwrap().clone(), Map::new());

            state.clear();
            assert_eq!(session.state().unwrap().clone(), state);
            assert_eq!(
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                "{}"
            );

            *session.state_mut().unwrap() = serde_json::from_str(
                r#"{"counter":144,"number":233,"user":{"age":37,"name":"Kobe"}}"#,
            )
            .unwrap();
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                r#"{"counter":144,"number":233,"user":{"age":37,"name":"Kobe"}}"#
            );

            assert_eq!(session.save().await.unwrap(), ());

            println!("{} ==>", i);
            dbg!(session);
            println!("{} <==", i);
        });
    }

    dbg!(store);
}
