use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};
use sessions::MemoryStore;
use sessions::Storable;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[test]
fn session_in_memory() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        no: u32,
        name: String,
    }

    let store = MemoryStore::new();

    let arc_store = Arc::new(store);

    let mut rt = Runtime::new().unwrap();

    let mut handlers = Vec::new();

    for i in 0..10 {
        let sid = format!("trek-{}", i);
        let store = arc_store.clone();

        handlers.push(rt.spawn(async move {
            // println!(" ========> {} <=========", i);
            // let session = Session::new(&sid, store);
            let session = store.get(&sid).await.unwrap();

            assert_eq!(session.sid(), sid);
            assert_eq!(session.fresh(), true);

            assert_eq!(session.set::<usize>("counter", i).unwrap(), None);
            assert_eq!(session.set("number", 233).unwrap(), None);
            assert_eq!(session.get::<usize>("counter").unwrap(), Some(i));
            assert_eq!(session.get::<u32>("number").unwrap(), Some(233));
            assert_eq!(
                session
                    .set(
                        "user",
                        User {
                            no: 23,
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
                            no: 24,
                            name: "Kobe".to_owned(),
                        }
                    )
                    .unwrap(),
                Some(User {
                    no: 23,
                    name: "Jordan".to_owned(),
                })
            );
            let user: Option<User> = session.get::<User>("user").unwrap();
            assert_eq!(
                user,
                Some(User {
                    no: 24,
                    name: "Kobe".to_owned(),
                })
            );

            let mut state = Map::new();
            state.insert("counter".to_owned(), json!(i));
            state.insert("number".to_owned(), json!(233));
            state.insert(
                "user".to_owned(),
                json!(User {
                    no: 24,
                    name: "Kobe".to_owned(),
                }),
            );
            assert_eq!(session.state().unwrap().clone(), state);
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"no":24,"name":"Kobe"}}}}"#,
                    i
                )
            );
            assert_eq!(
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"no":24,"name":"Kobe"}}}}"#,
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
                r#"{{"counter":{},"number":233,"user":{{"no":24,"name":"Kobe"}}}}"#,
                i
            ))
            .unwrap();
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"no":24,"name":"Kobe"}}}}"#,
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
        // dbg!(Arc::try_unwrap(arc_store).unwrap());
        // println!("--------------------------------------");

        for i in 0..10 {
            let sid = format!("trek-{}", i);
            let sess = arc_store.get(&sid).await;

            assert_eq!(sess.is_ok(), true);

            let session = sess.unwrap();

            assert_eq!(session.fresh(), false);

            let mut count = session.get::<usize>("counter").unwrap().unwrap();

            assert_eq!(count, i);

            count += 1;

            assert_eq!(session.set("index", count).unwrap(), None);

            assert_eq!(session.remove::<User>("user").is_ok(), true);
            assert_eq!(session.remove::<i32>("number").unwrap(), Some(233));

            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );

            assert_eq!(session.save().await.unwrap(), ());
        }

        // dbg!(Arc::try_unwrap(arc_store).unwrap());
    });
}
