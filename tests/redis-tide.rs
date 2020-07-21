use std::{convert::TryInto, str::FromStr, sync::Arc};

use async_trait::async_trait;
use cookie::Cookie;
use log::info;
use redis::Client as RedisClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sessions::{RedisStore, Session, SessionStatus, Storable};
use tide::{self, http, Middleware, Next, Request, Response};
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

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for SessionsMiddleware {
    async fn handle(
        &self,
        mut req: Request<State>,
        next: Next<'_, State>,
    ) -> tide::Result<Response> {
        let sid = req
            .cookie(SESSION_NAME)
            .map(|c| c.value().to_owned())
            .filter(|id| id.len() == 32)
            .unwrap_or_default();

        req.set_ext(self.store.get(&sid).await);

        Ok(next.run(req).await)
    }
}

trait RequestExt {
    fn session(&self) -> &Session;
}

impl<State: Send + Sync + 'static> RequestExt for Request<State> {
    fn session(&self) -> &Session {
        self.ext::<Session>().unwrap()
    }
}

#[async_std::test]
async fn tide_with_redis() -> Result<(), surf::Exception> {
    pretty_env_logger::init();

    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    let arc_store = Arc::new(RedisStore::new(
        Arc::new(RedisClient::open("redis://localhost/").unwrap()),
        "session:id:",
        60 * 5,
    ));

    let store_0 = arc_store.clone();
    let store_1 = arc_store.clone();

    let mut app = tide::new();

    app.middleware(SessionsMiddleware::new(store_0));

    app.at("/").get(|req: tide::Request<()>| async move {
        let session = req.session();
        if session.status().await == SessionStatus::Existed {
            let count = session.get::<usize>("count").await.unwrap_or_default() + 1;
            session.set("count", count).await;
            session.save().await;
            info!("User is logged in, {}.", count);
            Ok(json!(&session.state().await).into())
        } else {
            info!("User is not logged in.");
            let mut res = tide::Response::new(200);
            res.set_body("");
            Ok(res)
        }
    });

    app.at("/session")
        .post(|req: tide::Request<()>| async move {
            let session = req.session();
            let mut count = session.get::<usize>("count").await.unwrap_or_default();
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
                res.insert_cookie(Cookie::new(SESSION_NAME, session.id().await));
            }
            res.set_body(json!(&session.state().await));
            Ok(res)
        });

    app.at("/logout").post(|req: tide::Request<()>| async move {
        let session = req.session();
        if session.status().await == SessionStatus::Existed {
            let count = session.get::<usize>("count").await.unwrap_or_default() + 1;
            info!("User is logged in, {}.", count);
            session.set("count", count).await;
            info!("Session is destroyed.");
            session.destroy().await;
            let cookie = Cookie::build(SESSION_NAME, session.id().await)
                .max_age(Duration::seconds(-1))
                .finish();
            let mut res = tide::Response::new(200);
            res.insert_cookie(cookie);
            res.set_body(json!(&session.state().await));
            Ok(res)
        } else {
            info!("Session is not found.");
            Ok(tide::Response::new(403))
        }
    });

    async_std::task::sleep(Duration::milliseconds(100).try_into()?).await;

    // First visit home
    let req = http::Request::new(http::Method::Get, http::Url::parse("http://localhost")?);
    let mut res: http::Response = app.respond(req).await?;
    let buf = res.body_bytes().await?;
    assert_eq!(res.status(), 200);
    assert!(buf.is_empty());

    // First login
    let req = http::Request::new(
        http::Method::Post,
        http::Url::parse("http://localhost/session")?,
    );
    let mut res: http::Response = app.respond(req).await?;
    assert_eq!(res.status(), 200);
    let session_cookie = Cookie::from_str(
        res.header("SET-COOKIE")
            .and_then(|a| a.get(0))
            .map(|v| v.as_str())
            .unwrap_or_default(),
    )?;
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
    let mut req = http::Request::new(
        http::Method::Post,
        http::Url::parse("http://localhost/session")?,
    );
    req.insert_header("COOKIE", format!("{}={}", SESSION_NAME, sid));
    let mut res: http::Response = app.respond(req).await?;
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
    let mut req = http::Request::new(
        http::Method::Post,
        http::Url::parse("http://localhost/session")?,
    );
    req.insert_header("COOKIE", format!("{}={}", SESSION_NAME, sid));
    let mut res: http::Response = app.respond(req).await?;
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
    let mut req = http::Request::new(
        http::Method::Post,
        http::Url::parse("http://localhost/logout")?,
    );
    req.insert_header("COOKIE", format!("{}={}", SESSION_NAME, sid));
    let mut res: http::Response = app.respond(req).await?;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await?, "{}");

    // Second logout.
    let mut req = http::Request::new(
        http::Method::Post,
        http::Url::parse("http://localhost/logout")?,
    );
    req.insert_header("COOKIE", format!("{}={}", SESSION_NAME, sid));
    let mut res: http::Response = app.respond(req).await?;
    assert_eq!(res.status(), 403);
    assert_eq!(res.body_string().await?, "");

    // Three visit home.
    let req = http::Request::new(http::Method::Get, http::Url::parse("http://localhost")?);
    let mut res: http::Response = app.respond(req).await?;
    let buf = res.body_bytes().await?;
    assert_eq!(res.status(), 200);
    assert!(buf.is_empty());

    Ok(())
}
