use cookie::{Cookie, CookieJar};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_value};
use sessions::{MemoryStore, Session, SessionStatus, Storable};
use std::{
    convert::Infallible,
    io::Error,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use time::Duration;
use tokio::runtime::Runtime;
use warp::{
    http::{header, Response, StatusCode},
    hyper::{
        server::Server,
        service::{make_service_fn, service_fn},
        Body, Client, HeaderMap, Method, Request,
    },
    reject::{not_found, Reject},
    Filter, Rejection, Reply,
};

static SESSION_NAME: &str = "session.id";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: String,
    logged_in: bool,
    count: u32,
}

#[derive(Debug)]
struct MyError;

impl Reject for MyError {}

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type GenericResult<T> = std::result::Result<T, GenericError>;

// From https://github.com/http-rs/tide/blob/master/src/middleware/cookies.rs
#[derive(Debug, Clone)]
struct CookieData {
    content: Arc<RwLock<CookieJar>>,
}

impl CookieData {
    fn from_headers(headers: &HeaderMap, key: header::HeaderName) -> Self {
        let cookie_header = headers.get(key);
        let cookie_jar = cookie_header.and_then(|raw| {
            let mut jar = CookieJar::new();

            // as long as we have an ascii string this will start parsing the cookie
            if let Some(raw_str) = raw.to_str().ok() {
                raw_str
                    .split(';')
                    .try_for_each(|s| -> GenericResult<_> {
                        jar.add_original(Cookie::parse(s.trim().to_owned())?);
                        Ok(())
                    })
                    .ok()?;
            }

            Some(jar)
        });
        let content = Arc::new(RwLock::new(cookie_jar.unwrap_or_default()));

        CookieData { content }
    }
}

async fn respond(addr: SocketAddr, store: Arc<dyn Storable>) -> GenericResult<()> {
    // First visit home.
    let req = Request::builder()
        .uri(format!("http://{}/", addr))
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 200);
    let buf = hyper::body::to_bytes(res).await?;
    assert!(buf.is_empty());

    // First Login.
    let req = Request::builder()
        .uri(format!("http://{}/{}", addr, "session"))
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 200);
    let cookie_data = CookieData::from_headers(res.headers(), header::SET_COOKIE);
    let content = cookie_data.content.read().unwrap();
    let id = content.get(SESSION_NAME).map(|c| c.value());
    let buf = hyper::body::to_bytes(res).await?;
    assert!(!buf.is_empty());
    let user = from_slice::<User>(&buf)?;
    assert_eq!(id, Some(user.id.as_str()));
    assert_eq!(0, user.count);

    let session = store.get(id.unwrap()).await?;
    assert_eq!(session.status()?, SessionStatus::Created);
    assert_eq!(
        to_value(user)?.as_object().map(|m| m.to_owned()).unwrap(),
        session.state()?
    );

    // Second Login.
    let req = Request::builder()
        .uri(format!("http://{}/{}", addr, "session"))
        .header(
            header::COOKIE,
            cookie_data
                .content
                .read()
                .unwrap()
                .get(SESSION_NAME)
                .unwrap()
                .encoded()
                .to_string(),
        )
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 200);
    let buf = hyper::body::to_bytes(res).await?;
    assert!(!buf.is_empty());
    let user = from_slice::<User>(&buf)?;
    assert_eq!(id, Some(user.id.as_str()));
    assert_eq!(1, user.count);

    let session = store.get(id.unwrap()).await?;
    assert_eq!(session.status()?, SessionStatus::Existed);
    assert_eq!(
        to_value(user)?.as_object().map(|m| m.to_owned()).unwrap(),
        session.state()?
    );

    // Second visit home.
    let req = Request::builder()
        .uri(format!("http://{}/", addr))
        .header(
            header::COOKIE,
            cookie_data
                .content
                .read()
                .unwrap()
                .get(SESSION_NAME)
                .unwrap()
                .encoded()
                .to_string(),
        )
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 200);
    let buf = hyper::body::to_bytes(res).await?;
    assert!(!buf.is_empty());
    let user = from_slice::<User>(&buf)?;
    assert_eq!(id, Some(user.id.as_str()));
    assert_eq!(2, user.count);

    let session = store.get(id.unwrap()).await?;
    assert_eq!(session.status()?, SessionStatus::Existed);
    assert_eq!(
        to_value(user)?.as_object().map(|m| m.to_owned()).unwrap(),
        session.state()?
    );

    // First logout.
    let req = Request::builder()
        .uri(format!("http://{}/{}", addr, "logout"))
        .header(
            header::COOKIE,
            cookie_data
                .content
                .read()
                .unwrap()
                .get(SESSION_NAME)
                .unwrap()
                .encoded()
                .to_string(),
        )
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 200);
    let buf = hyper::body::to_bytes(res).await?;
    assert!(!buf.is_empty());
    let user = from_slice::<User>(&buf)?;
    assert_eq!(id, Some(user.id.as_str()));
    assert_eq!(3, user.count);

    let session = store.get(id.unwrap()).await?;
    assert_eq!(session.status()?, SessionStatus::Created);
    assert!(session.state()?.is_empty());

    // Second logout.
    let req = Request::builder()
        .uri(format!("http://{}/{}", addr, "logout"))
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 403);
    let buf = hyper::body::to_bytes(res).await?;
    assert!(buf.is_empty());

    // Three visit home.
    let req = Request::builder()
        .uri(format!("http://{}/", addr))
        .body(Body::empty())
        .unwrap();
    let res = Client::new().request(req).await?;
    assert_eq!(res.status(), 200);
    let buf = hyper::body::to_bytes(res).await?;
    assert!(buf.is_empty());

    dbg!(id.unwrap());
    Ok(())
}

async fn home(session: Session) -> Result<Response<Body>, Error> {
    let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

    Ok(if session.status()? == SessionStatus::Existed {
        let count = session.get::<usize>("count")?.unwrap_or_else(|| 0) + 1;
        session.set("count", count)?;
        session.save().await?;
        info!("User is logged in, {}.", count);
        builder.body(Body::from(serde_json::to_vec(&session.state()?)?))
    } else {
        info!("User is not logged in.");
        builder.body(Body::empty())
    }
    .unwrap())
}

async fn login(session: Session) -> Result<Response<Body>, Error> {
    let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

    let mut count = session.get::<usize>("count")?.unwrap_or_else(|| 0);

    Ok(if session.status()? == SessionStatus::Existed {
        count += 1;
        session.set("count", count)?;
        session.save().await?;
        info!("User is logged in, {}.", count);
        builder
    } else {
        session.set("logged_in", true)?;
        session.set("count", count)?;
        session.save().await?;
        info!("User is logged in, {}.", count);
        builder.header(
            header::SET_COOKIE,
            Cookie::new(SESSION_NAME, session.id()?)
                .encoded()
                .to_string(),
        )
    }
    .body(Body::from(serde_json::to_vec(&session.state()?)?))
    .unwrap())
}

async fn logout(session: Session) -> Result<Response<Body>, Error> {
    let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

    Ok(if session.status()? == SessionStatus::Existed {
        let count = session.get::<usize>("count")?.unwrap_or_else(|| 0) + 1;
        info!("User is logged in, {}", count);
        session.set("count", count)?;
        info!("Session is destroyed");
        session.destroy().await?;
        let cookie = Cookie::build(SESSION_NAME, session.id()?)
            .max_age(Duration::seconds(-1))
            .finish();
        builder
            .header(header::SET_COOKIE, cookie.encoded().to_string())
            .body(Body::from(serde_json::to_vec(&session.state()?)?))
    } else {
        info!("Session is not found");
        builder.status(403).body(Body::empty())
    }
    .unwrap())
}

#[test]
fn hyper_and_warp_with_memory() {
    pretty_env_logger::init();

    warp_with_memory();

    hyper_with_memory();
}

fn warp_with_memory() {
    fn with_session(
        store: Arc<dyn Storable>,
    ) -> impl Filter<Extract = (Session,), Error = Rejection> + Clone {
        warp::any()
            .map(move || store.clone())
            .and(warp::filters::cookie::optional(SESSION_NAME))
            .and_then(
                move |store: Arc<dyn Storable>, cookie: Option<String>| async move {
                    let sid = cookie
                        .filter(|id| id.len() == 32)
                        .unwrap_or_else(|| "".to_owned());
                    match store.get(&sid).await {
                        Ok(session) => Ok(session),
                        Err(err) => {
                            dbg!(err);
                            Err(not_found())
                        }
                    }
                },
            )
    }

    async fn home_wrap(session: Session) -> Result<impl Reply, Rejection> {
        home(session)
            .await
            .map_err(|_| warp::reject::custom(MyError))
    }

    async fn login_wrap(session: Session) -> Result<impl Reply, Rejection> {
        login(session)
            .await
            .map_err(|_| warp::reject::custom(MyError))
    }

    async fn logout_wrap(session: Session) -> Result<impl Reply, Rejection> {
        logout(session)
            .await
            .map_err(|_| warp::reject::custom(MyError))
    }

    let mut rt = Runtime::new().unwrap();

    let addr: std::net::SocketAddr = "127.0.0.1:1337".parse().unwrap();

    let store = MemoryStore::new();

    let arc_store = Arc::new(store);

    let store_1 = arc_store.clone();

    // GET `/`
    let home_route = warp::path::end()
        .and(warp::get())
        .and(with_session(arc_store.clone()))
        .and_then(home_wrap);

    // POST `/session`
    let login_route = warp::path!("session")
        .and(warp::post())
        .and(with_session(arc_store.clone()))
        .and_then(login_wrap);

    // POST `/logout`
    let logout_route = warp::path!("logout")
        .and(warp::post())
        .and(with_session(arc_store.clone()))
        .and_then(logout_wrap);

    let routes = home_route.or(login_route).or(logout_route);

    rt.spawn(async move {
        warp::serve(routes).run(addr).await;
    });

    rt.block_on(async move {
        for _ in 0..10 {
            let _ = respond(addr, store_1.clone()).await;
        }
    });
}

fn hyper_with_memory() {
    async fn serve(addr: SocketAddr, store: Arc<dyn Storable>) -> GenericResult<()> {
        let new_service = make_service_fn(move |_| {
            let store = store.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let store = store.clone();
                    async move {
                        async fn process(
                            mut req: Request<Body>,
                            store: Arc<dyn Storable>,
                        ) -> Result<Response<Body>, std::io::Error> {
                            let cookie_data =
                                CookieData::from_headers(req.headers(), http::header::COOKIE);

                            let sid = cookie_data
                                .content
                                .read()
                                .unwrap()
                                .get(SESSION_NAME)
                                .cloned()
                                .map(|c| c.value().to_owned())
                                .filter(|id| id.len() == 32)
                                .unwrap_or_else(|| "".to_owned());

                            req.extensions_mut().insert(cookie_data);
                            // req.extensions_mut().insert(session);

                            let session = store.get(&sid).await?;

                            match (req.method(), req.uri().path()) {
                                (&Method::GET, "/") => home(session).await,
                                (&Method::POST, "/session") => login(session).await,
                                (&Method::POST, "/logout") => logout(session).await,
                                _ => {
                                    // Return 404 not found response.
                                    Ok(Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(Body::empty())
                                        .unwrap())
                                }
                            }
                        }

                        process(req, store).await.or_else(|e| {
                            Response::builder()
                                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(e.to_string()))
                        })
                    }
                }))
            }
        });

        let server = Server::bind(&addr).serve(new_service);

        info!("Listening on http://{}", addr);

        server.await?;

        Ok(())
    }

    let mut rt = Runtime::new().unwrap();

    let addr = "127.0.0.1:1338".parse().unwrap();

    let store = MemoryStore::new();

    let arc_store = Arc::new(store);
    let store_0 = arc_store.clone();
    let store_1 = arc_store.clone();

    rt.spawn(async move {
        let _ = serve(addr, store_0).await;
    });

    rt.block_on(async move {
        for _ in 0..10 {
            let _ = respond(addr, store_1.clone()).await;
        }
    });
}
