use std::sync::{Arc, RwLock};

use smol::block_on;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};

use sessions::{MemoryStore, SessionStatus, Storable};

#[test]
fn session_in_memory() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        name: String,
        no: u32,
    }

    let sids = Arc::new(RwLock::new(Vec::new()));

    let store = MemoryStore::new();

    let arc_store = Arc::new(store);

    let mut handlers = Vec::new();

    block_on(async move {
        for i in 0..10 {
            let id = format!("trek-{}", i);
            let store = arc_store.clone();
            let sids = sids.clone();

            handlers.push(smol::Task::spawn(async move {
                println!(" ========> {} <=========", i);
                // let session = Session::new(&id, store);
                let session = store.get(&id).await;

                assert_eq!(session.id().await, "".to_owned());
                assert_eq!(session.status().await, SessionStatus::Created);

                assert_eq!(session.set::<usize>("counter", i).await, None);
                assert_eq!(session.set("number", 233).await, None);
                assert_eq!(session.get::<usize>("counter").await, Some(i));
                assert_eq!(session.get::<u32>("number").await, Some(233));
                assert_eq!(
                    session
                        .set(
                            "user",
                            User {
                                name: "Jordan".to_owned(),
                                no: 23,
                            }
                        )
                        .await,
                    None
                );
                assert_eq!(
                    session
                        .set(
                            "user",
                            User {
                                name: "Kobe".to_owned(),
                                no: 24,
                            }
                        )
                        .await,
                    Some(User {
                        name: "Jordan".to_owned(),
                        no: 23,
                    })
                );
                let user: Option<User> = session.get::<User>("user").await;
                assert_eq!(
                    user,
                    Some(User {
                        name: "Kobe".to_owned(),
                        no: 24,
                    })
                );

                let mut state = Map::new();
                state.insert("counter".to_owned(), json!(i));
                state.insert("number".to_owned(), json!(233));
                state.insert(
                    "user".to_owned(),
                    json!(User {
                        name: "Kobe".to_owned(),
                        no: 24,
                    }),
                );
                assert_eq!(session.state().await, state);
                assert_eq!(
                    serde_json::to_string(&state).unwrap(),
                    format!(
                        r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                        i
                    )
                );
                assert_eq!(
                    serde_json::to_string(&session.state().await).unwrap(),
                    format!(
                        r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                        i
                    )
                );

                assert_eq!(session.remove("number").await, Some(json!(233)));
                assert_eq!(session.remove::<f32>("counter").await, Some(i as f32));
                assert_eq!(session.get::<u32>("counter").await, None);
                assert_eq!(session.remove::<usize>("counter").await, None);

                state.remove("number");
                state.remove("counter");
                assert_eq!(session.state().await, state);

                assert_eq!(session.clear().await, ());
                assert_eq!(session.state().await, Map::new());

                state.clear();
                assert_eq!(session.state().await, state);
                assert_eq!(serde_json::to_string(&session.state().await).unwrap(), "{}");

                session
                    .set_state(
                        serde_json::from_str(&format!(
            // *session.state_mut().unwrap() = serde_json::from_str(&format!(
                r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                i
            ))
                        .unwrap(),
                    )
                    .await;

                assert_eq!(
                    to_string(&session.state().await).unwrap(),
                    format!(
                        r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                        i
                    )
                );

                assert_eq!(session.save().await, true);

                assert_eq!(session.id().await.len(), 32);

                let id = session.id().await;
                sids.write().unwrap().push((i, id));

                println!("{} ==>", i);
                dbg!(session);
                println!("{} <==", i);
            }));
        }

        join_all(handlers).await;
        println!("--------------------------------------");
        dbg!(&arc_store);
        println!("--------------------------------------");

        for (i, sid) in &*sids.read().unwrap() {
            let session = arc_store.get(&sid).await;

            assert_eq!(session.status().await, SessionStatus::Existed);

            let mut count = session.get::<usize>("counter").await.unwrap();

            assert_eq!(count, *i);

            count += 1;

            assert_eq!(session.set("index", count).await, None);

            assert_eq!(session.remove::<User>("user").await.is_some(), true);
            assert_eq!(session.remove::<i32>("number").await, Some(233));

            assert_eq!(
                to_string(&session.state().await.clone()).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );

            assert_eq!(session.save().await, true);

            assert_eq!(session.destroy().await, true);

            assert_eq!(session.status().await, SessionStatus::Destroyed);

            println!("{} ==>", i);
            dbg!(session);
            println!("{} <==", i);
        }

        dbg!(Arc::try_unwrap(arc_store).unwrap());
    });
}
