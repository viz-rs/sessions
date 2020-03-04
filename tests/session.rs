use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};
#[cfg(feature = "memory")]
use sessions::MemoryStore;
use sessions::{Session, Storable};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[test]
#[cfg(feature = "memory")]
fn session() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        age: u32,
        name: String,
    }

    let store = MemoryStore::new();

    let arc_store = Arc::new(store);

    let mut rt = Runtime::new().unwrap();

    let mut handlers = Vec::new();

    for i in 0..10 {
        let name = format!("trek-{}", i);
        let store = arc_store.clone();

        handlers.push(rt.spawn(async move {
            // println!(" ========> {} <=========", i);
            // let session = Session::new(&name, store);
            let session = store.create(&name);

            assert_eq!(session.name(), name);

            assert_eq!(session.set::<usize>("counter", i).unwrap(), None);
            assert_eq!(session.set("number", 233).unwrap(), None);
            assert_eq!(session.get::<usize>("counter").unwrap(), Some(i));
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
            state.insert("counter".to_owned(), json!(i));
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
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"age":37,"name":"Kobe"}}}}"#,
                    i
                )
            );
            assert_eq!(
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"age":37,"name":"Kobe"}}}}"#,
                    i
                )
            );

            assert_eq!(session.remove("number").unwrap(), Some(json!(233)));
            assert_eq!(session.remove::<f32>("counter").unwrap(), Some(i as f32));
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

            *session.state_mut().unwrap() = serde_json::from_str(&format!(
                r#"{{"counter":{},"number":233,"user":{{"age":37,"name":"Kobe"}}}}"#,
                i
            ))
            .unwrap();
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"age":37,"name":"Kobe"}}}}"#,
                    i
                )
            );

            assert_eq!(session.save().await.unwrap(), ());

            // println!("{} ==>", i);
            // dbg!(session);
            // println!("{} <==", i);
        }));
    }

    rt.block_on(async {
        join_all(handlers).await;
        // println!("--------------------------------------");
        dbg!(Arc::try_unwrap(arc_store).unwrap());
        // println!("--------------------------------------");
    });
}
