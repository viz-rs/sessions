use async_std::{fs, task};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map};
use sessions::{FilesystemStore, SessionStatus, Storable};
use std::future::Future;
use std::{
    env,
    sync::{Arc, RwLock},
};

struct Runtime {
    count: usize,
}

impl Runtime {
    fn new() -> std::io::Result<Self> {
        Ok(Self { count: 0 })
    }

    fn spawn<F, T>(&self, future: F) -> task::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        task::spawn(future)
    }

    fn block_on<F, T>(&mut self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.count += 1;
        task::block_on(future)
    }
}

#[test]
fn session_in_filesystem() {
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
            let session = store.get(&id).await;
            // store.remove(&id).await;

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
            let session = arc_store.get(&sid).await;

            assert_eq!(session.status().await, SessionStatus::Existed);

            let mut count = session.get::<usize>("counter").await.unwrap();

            assert_eq!(count, *i);

            count += 1;

            assert_eq!(session.set("index", count).await, None);

            assert_eq!(session.remove::<User>("user").await.is_some(), true);
            assert_eq!(session.remove::<i32>("number").await, Some(233));

            assert_eq!(
                to_string(&session.state().await).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );

            assert_eq!(session.save().await, true);

            let session = arc_store.get(&sid).await;
            assert_eq!(session.status().await, SessionStatus::Existed);
            assert_eq!(
                to_string(&session.state().await).unwrap(),
                format!(r#"{{"counter":{},"index":{}}}"#, count - 1, count)
            );

            assert_eq!(session.destroy().await, true);

            assert_eq!(session.status().await, SessionStatus::Destroyed);
        }

        // dbg!(Arc::try_unwrap(arc_store).unwrap());

        let _ = fs::remove_dir_all(path.clone()).await;
    });
}
