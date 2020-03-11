use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};
use sessions::{FilesystemStore, SessionStatus, Storable};
use std::{
    env,
    sync::{Arc, RwLock},
};
use tokio::{fs, runtime::Runtime};

#[test]
fn session_in_filesystem_with_tokio() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        name: String,
        no: u32,
    }

    let path = env::current_dir().unwrap().join("target").join("sessions");
    let store = FilesystemStore::new(path.clone());

    let sids = Arc::new(RwLock::new(Vec::new()));

    let arc_store = Arc::new(store);

    let mut rt = Runtime::new().unwrap();

    let mut handlers = Vec::new();

    for i in 0..10 {
        let id = format!("trek-{}", i);
        let store = arc_store.clone();
        let sids = sids.clone();

        handlers.push(rt.spawn(async move {
            // println!(" ========> {} <=========", i);
            // let session = Session::new(&id, store);
            let session = store.get(&id).await.unwrap();
            // store.remove(&id).await;

            assert_eq!(session.id().unwrap(), "".to_owned());
            assert_eq!(session.status().unwrap(), SessionStatus::Created);

            assert_eq!(session.set::<usize>("counter", i).unwrap(), None);
            assert_eq!(session.set("number", 233).unwrap(), None);
            assert_eq!(session.get::<usize>("counter").unwrap(), Some(i));
            assert_eq!(session.get::<u32>("number").unwrap(), Some(233));
            assert_eq!(
                session
                    .set(
                        "user",
                        User {
                            name: "Jordan".to_owned(),
                            no: 23,
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
                            name: "Kobe".to_owned(),
                            no: 24,
                        }
                    )
                    .unwrap(),
                Some(User {
                    name: "Jordan".to_owned(),
                    no: 23,
                })
            );
            let user: Option<User> = session.get::<User>("user").unwrap();
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
            assert_eq!(session.state().unwrap().clone(), state);
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                    i
                )
            );
            assert_eq!(
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
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

            // *session.state_mut().unwrap() = serde_json::from_str(&format!(
            //     r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
            //     i
            // ))
            // .unwrap();
            let _ = session.set_state(
                serde_json::from_str(&format!(
            // *session.state_mut().unwrap() = serde_json::from_str(&format!(
                r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                i
            ))
                .unwrap(),
            );
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"name":"Kobe","no":24}}}}"#,
                    i
                )
            );

            assert_eq!(session.save().await.unwrap(), ());

            assert_eq!(session.id().unwrap().len(), 32);

            sids.write().unwrap().push((i, session.id().unwrap()));

            // println!("{} ==>", i);
            // dbg!(session);
            // println!("{} <==", i);
        }));
    }

    rt.block_on(async {
        let _ = fs::create_dir(path.clone()).await;

        join_all(handlers).await;
        // // println!("--------------------------------------");
        // // dbg!(Arc::try_unwrap(arc_store).unwrap());
        // // println!("--------------------------------------");

        for (i, sid) in &*sids.read().unwrap() {
            let sess = arc_store.get(&sid).await;

            assert_eq!(sess.is_ok(), true);

            let session = sess.unwrap();

            assert_eq!(session.status().unwrap(), SessionStatus::Existed);

            let mut count = session.get::<usize>("counter").unwrap().unwrap();

            assert_eq!(count, *i);

            count += 1;

            assert_eq!(session.set("index", count).unwrap(), None);

            assert_eq!(session.remove::<User>("user").is_ok(), true);
            assert_eq!(session.remove::<i32>("number").unwrap(), Some(233));

            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );

            assert_eq!(session.save().await.unwrap(), ());

            let sess = arc_store.get(&sid).await;
            assert_eq!(sess.is_ok(), true);
            let session = sess.unwrap();
            assert_eq!(session.status().unwrap(), SessionStatus::Existed);
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );

            let _ = session.destroy().await;

            assert_eq!(session.status().unwrap(), SessionStatus::Destroyed);
        }

        // dbg!(Arc::try_unwrap(arc_store).unwrap());

        let _ = fs::remove_dir_all(path.clone()).await;
    });
}
