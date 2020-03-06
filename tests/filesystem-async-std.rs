use async_std::{fs, task};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};
use sessions::FilesystemStore;
use sessions::Storable;
use std::env;
use std::sync::Arc;

#[test]
fn session_in_filesystem_with_async_std() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        age: u32,
        sid: String,
    }

    let path = env::current_dir().unwrap().join("target").join("sessions");
    let store = FilesystemStore::new(path.clone());

    let arc_store = Arc::new(store);

    let mut handlers = Vec::new();

    for i in 0..10 {
        let sid = format!("trek-{}", i);
        let store = arc_store.clone();

        handlers.push(task::spawn(async move {
            // println!(" ========> {} <=========", i);
            // let session = Session::new(&sid, store);
            let session = store.get(&sid).await.unwrap();
            // store.remove(&sid).await;

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
                            age: 23,
                            sid: "Jordan".to_owned(),
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
                            sid: "Kobe".to_owned(),
                        }
                    )
                    .unwrap(),
                Some(User {
                    age: 23,
                    sid: "Jordan".to_owned(),
                })
            );
            let user: Option<User> = session.get::<User>("user").unwrap();
            assert_eq!(
                user,
                Some(User {
                    age: 37,
                    sid: "Kobe".to_owned(),
                })
            );

            let mut state = Map::new();
            state.insert("counter".to_owned(), json!(i));
            state.insert("number".to_owned(), json!(233));
            state.insert(
                "user".to_owned(),
                json!(User {
                    age: 37,
                    sid: "Kobe".to_owned(),
                }),
            );
            assert_eq!(session.state().unwrap().clone(), state);
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"age":37,"sid":"Kobe"}}}}"#,
                    i
                )
            );
            assert_eq!(
                serde_json::to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"age":37,"sid":"Kobe"}}}}"#,
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
                r#"{{"counter":{},"number":233,"user":{{"age":37,"sid":"Kobe"}}}}"#,
                i
            ))
            .unwrap();
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(
                    r#"{{"counter":{},"number":233,"user":{{"age":37,"sid":"Kobe"}}}}"#,
                    i
                )
            );

            assert_eq!(session.save().await.unwrap(), ());

            // println!("{} ==>", i);
            // dbg!(session);
            // println!("{} <==", i);
        }));
    }

    task::block_on(async {
        let _ = fs::create_dir(path.clone()).await;

        join_all(handlers).await;
        // // println!("--------------------------------------");
        // // dbg!(Arc::try_unwrap(arc_store).unwrap());
        // // println!("--------------------------------------");

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

            let sess = arc_store.get(&sid).await;
            assert_eq!(sess.is_ok(), true);
            let session = sess.unwrap();
            assert_eq!(session.fresh(), false);
            assert_eq!(
                to_string(&session.state().unwrap().clone()).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );
        }

        // dbg!(Arc::try_unwrap(arc_store).unwrap());

        let _ = fs::remove_dir_all(path.clone()).await;
    });
}
