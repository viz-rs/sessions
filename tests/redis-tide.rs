use async_std::{prelude::*, task};
use cookie::Cookie;
use futures::future::BoxFuture;
use log::info;
use redis::Client as RedisClient;
use serde::{Deserialize, Serialize};
use sessions::{RedisStore, Session, SessionStatus, Storable};
use std::{convert::TryInto, str::FromStr, sync::Arc};
use tide::{self, Middleware, Next, Request, Response};
use time::Duration;

static SESSION_NAME: &str = "session.id";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    logged_in: bool,
    count: u32,
}

#[derive(Debug, Clone)]
struct SessionsMiddleware {
    store: Arc<RedisStore>,
}

impl Default for SessionsMiddleware {
    fn default() -> Self {
        Self {
            store: Arc::default(),
        }
    }
}

impl SessionsMiddleware {
    /// Creates a new SessionsMiddleware.
    pub fn new(store: Arc<RedisStore>) -> Self {
        Self { store }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for SessionsMiddleware {
    fn handle<'a>(
        &'a self,
        mut ctx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let sid = ctx
                .cookie(SESSION_NAME)
                .map(|c| c.value().to_owned())
                .filter(|id| id.len() == 32)
                .unwrap_or_else(|| "".to_owned());

            ctx = ctx.set_local(self.store.get(&sid).await);

            next.run(ctx).await
        })
    }
}

trait RequestExt {
    fn session(&self) -> &Session;
}

impl<State: Send + Sync + 'static> RequestExt for Request<State> {
    fn session(&self) -> &Session {
        self.local::<Session>().unwrap()
    }
}

#[test]
fn tide_with_redis() -> Result<(), surf::Exception> {
    pretty_env_logger::init();

    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    let arc_store = Arc::new(RedisStore::new(
        RedisClient::open("redis://127.0.0.1/").unwrap(),
        "session:id:",
        60 * 5,
    ));

    let store_0 = arc_store.clone();
    let store_1 = arc_store.clone();

    task::block_on(async {
        let server = task::spawn(async move {
            let mut app = tide::new();

            app.middleware(SessionsMiddleware::new(store_0));

            app.at("/").get(|req: tide::Request<()>| async move {
                let session = req.session();
                if session.status().await == SessionStatus::Existed {
                    let count = session.get::<usize>("count").await.unwrap_or_else(|| 0) + 1;
                    session.set("count", count).await;
                    session.save().await;
                    info!("User is logged in, {}.", count);
                    tide::Response::new(200)
                        .body_json(&session.state().await)
                        .unwrap()
                } else {
                    info!("User is not logged in.");
                    tide::Response::new(200).body_string("".to_owned())
                }
            });

            app.at("/session")
                .post(|req: tide::Request<()>| async move {
                    let session = req.session();
                    let mut count = session.get::<usize>("count").await.unwrap_or_else(|| 0);
                    let mut res = tide::Response::new(200);
                    if session.status().await == SessionStatus::Existed {
                        count += 1;
                        session.set("count", count).await;
                        session.save().await;
                        info!("User is logged in, {}.", count);
                    } else {
                        session.set("logged_in", true).await;
                        session.set("count", count).await;
                        session.save().await;
                        info!("User is logged in, {}.", count);
                        res.set_cookie(Cookie::new(SESSION_NAME, session.id().await));
                    }
                    res.body_json(&session.state().await).unwrap()
                });

            app.at("/logout")
                .post(|req: tide::Request<()>| async move {
                    let session = req.session();
                    if session.status().await == SessionStatus::Existed {
                        let count = session.get::<usize>("count").await.unwrap_or_else(|| 0) + 1;
                        info!("User is logged in, {}.", count);
                        session.set("count", count).await;
                        info!("Session is destroyed.");
                        session.destroy().await;
                        let cookie = Cookie::build(SESSION_NAME, session.id().await)
                            .max_age(Duration::seconds(-1))
                            .finish();
                        let mut res = tide::Response::new(200);
                        res.set_cookie(cookie);
                        res.body_json(&session.state().await).unwrap()
                    } else {
                        info!("Session is not found.");
                        tide::Response::new(403)
                    }
                });

            app.listen("localhost:8082").await?;

            Result::<(), surf::Exception>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::milliseconds(100).try_into()?).await;

            // First visit home
            let mut res = surf::get("http://localhost:8082").await?;
            let buf = res.body_bytes().await?;
            assert_eq!(res.status(), 200);
            assert!(buf.is_empty());

            // First login
            let mut res = surf::post("http://localhost:8082/session").await?;
            assert_eq!(res.status(), 200);
            let session_cookie = Cookie::from_str(res.header("SET-COOKIE").unwrap_or_else(|| ""))?;
            let sid = session_cookie.value();
            assert_eq!(sid.len(), 32);
            let user: User = res.body_json().await?;
            assert_eq!(true, user.logged_in);
            assert_eq!(0, user.count);

            let session = store_1.get(sid).await;
            assert_eq!(session.status().await, SessionStatus::Existed);
            assert_eq!(
                serde_json::to_value(user)?
                    .as_object()
                    .map(|m| m.to_owned())
                    .unwrap(),
                session.state().await
            );

            // Second Login.
            let mut res = surf::post("http://localhost:8082/session")
                .set_header("COOKIE", format!("{}={}", SESSION_NAME, sid))
                .await?;
            assert_eq!(res.status(), 200);
            let user: User = res.body_json().await?;
            assert_eq!(true, user.logged_in);
            assert_eq!(1, user.count);

            let session = store_1.get(sid).await;
            assert_eq!(session.status().await, SessionStatus::Existed);
            assert_eq!(
                serde_json::to_value(user)?
                    .as_object()
                    .map(|m| m.to_owned())
                    .unwrap(),
                session.state().await
            );

            // Second visit home.
            let mut res = surf::post("http://localhost:8082/session")
                .set_header("COOKIE", format!("{}={}", SESSION_NAME, sid))
                .await?;
            assert_eq!(res.status(), 200);
            let user: User = res.body_json().await?;
            assert_eq!(true, user.logged_in);
            assert_eq!(2, user.count);

            let session = store_1.get(sid).await;
            assert_eq!(session.status().await, SessionStatus::Existed);
            assert_eq!(
                serde_json::to_value(user)?
                    .as_object()
                    .map(|m| m.to_owned())
                    .unwrap(),
                session.state().await
            );

            // First logout.
            let mut res = surf::post("http://localhost:8082/logout")
                .set_header("COOKIE", format!("{}={}", SESSION_NAME, sid))
                .await?;
            assert_eq!(res.status(), 200);
            assert_eq!(res.body_string().await?, "{}");

            // Second logout.
            let mut res = surf::post("http://localhost:8082/logout")
                .set_header("COOKIE", format!("{}={}", SESSION_NAME, sid))
                .await?;
            assert_eq!(res.status(), 403);
            assert_eq!(res.body_string().await?, "");

            // Three visit home.
            let mut res = surf::get("http://localhost:8082").await?;
            let buf = res.body_bytes().await?;
            assert_eq!(res.status(), 200);
            assert!(buf.is_empty());

            Ok(())
        });

        server.race(client).await
    })
}
