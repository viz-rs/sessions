use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Map};
use sessions::session::*;
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use std::thread;

#[test]
fn session() {
    // #[derive(Clone, Debug, PartialEq)]
    // struct MyStore {
    //     values: HashMap<String, String>,
    // }

    // impl MyStore {
    //     fn new() -> Self {
    //         Self {
    //             values: HashMap::new(),
    //         }
    //     }
    // }

    // impl Storable for MyStore {
    //     fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
    //         if let Some(value) = self.values.get(key) {
    //             Ok(Some(serde_json::from_str(value)?))
    //         } else {
    //             Ok(None)
    //         }
    //     }

    //     fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), Error> {
    //         self.values
    //             .insert(key.to_owned(), serde_json::to_string(&value)?);

    //         Ok(())
    //     }

    //     // fn save<S: Sessionable<Self>>(&mut self, session: S) {}
    // }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        age: u32,
        name: String,
    }

    // let store = MyStore::new();

    // let store = Arc::new(store);

    for i in 0..10 {
        // let name = format!("trek-{}", i);
        // let store = store.clone();

        thread::spawn(move || {
            println!(" ========> {} <=========", i);
            // let mut session = Session::new(&name, store);
            let session = Session::new();

            // assert_eq!(session.name(), &name);

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

            session.clear();
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
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                r#"{"counter":144,"number":233,"user":{"age":37,"name":"Kobe"}}"#
            );

            assert_eq!(
                session.state().unwrap().clone(),
                Session::from(session.state().unwrap().clone())
                    .state()
                    .unwrap()
                    .clone()
            );

            println!("{} ==>", i);
            dbg!(session);
            println!("{} <==", i);
        })
        .join()
        .unwrap();
    }

    // dbg!(store);
}
