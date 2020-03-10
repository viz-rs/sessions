use cookie::{Cookie, CookieJar};
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, HeaderMap, Method, Request, Response, Server, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_value};
use sessions::{MemoryStore, SessionStatus, Storable};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::runtime::Runtime;

static NOTFOUND: &[u8] = b"Not Found";
static SESSION_NAME: &str = "session.id";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: String,
    logged_in: bool,
    count: u32,
}

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
                    .try_for_each(|s| -> Result<_> {
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

async fn home(req: Request<Body>, store: Arc<dyn Storable>) -> Result<Response<Body>> {
    let cookies = req
        .extensions()
        .get::<CookieData>()
        .unwrap()
        .content
        .read()
        .unwrap()
        .clone();

    let cookie: Option<&str> = cookies.get(SESSION_NAME).map(|c| c.value());

    let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

    let res = if let Some(id) = cookie {
        let session = store.get(&id).await?;
        if session.status()? == SessionStatus::Existed {
            let count = session.get::<usize>("count")?.unwrap_or_else(|| 0) + 1;
            println!("User is logged in, {}.", count);
            session.set("count", count)?;
            session.save().await?;
            builder.body(Body::from(serde_json::to_vec(&session.state()?)?))
        } else {
            println!("User is not logged in.");
            builder.body(Body::empty())
        }
    } else {
        println!("User is not logged in.");
        builder.body(Body::empty())
    }
    .unwrap();

    Ok(res)
}

async fn login(req: Request<Body>, store: Arc<dyn Storable>) -> Result<Response<Body>> {
    let cookies = req
        .extensions()
        .get::<CookieData>()
        .unwrap()
        .content
        .read()
        .unwrap()
        .clone();

    let cookie: Option<&str> = cookies.get(SESSION_NAME).map(|c| c.value());

    let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

    let res = if let Some(id) = cookie {
        let session = store.get(&id).await?;
        let count = session.get::<usize>("count")?.unwrap_or_else(|| 0) + 1;
        println!("User is logged in, {}.", count);
        session.set("count", count)?;
        session.save().await?;
        builder.body(Body::from(serde_json::to_vec(&session.state()?)?))
    } else {
        let id = nanoid!();
        let session = store.get(&id).await?;
        let count = 0;
        println!("User is logged in, {}.", count);
        session.set("logged_in", true)?;
        session.set("id", id.clone())?;
        session.set("count", count)?;
        session.save().await?;
        builder
            .header(
                header::SET_COOKIE,
                Cookie::new(SESSION_NAME, id).encoded().to_string(),
            )
            .body(Body::from(serde_json::to_vec(&session.state()?)?))
    }
    .unwrap();

    Ok(res)
}

async fn logout(req: Request<Body>, store: Arc<dyn Storable>) -> Result<Response<Body>> {
    let cookies = req
        .extensions()
        .get::<CookieData>()
        .unwrap()
        .content
        .read()
        .unwrap()
        .clone();

    let cookie: Option<&str> = cookies.get(SESSION_NAME).map(|c| c.value());

    let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

    let res = if let Some(id) = cookie {
        let session = store.get(&id).await?;
        if session.status()? == SessionStatus::Existed {
            let count = session.get::<usize>("count")?.unwrap_or_else(|| 0) + 1;
            println!("User is logged in, {}", count);
            session.set("count", count)?;
            println!("Session is destroyed");
            session.destroy().await?;
            builder.body(Body::from(serde_json::to_vec(&session.state()?)?))
        } else {
            println!("Session is not found");
            builder.status(403).body(Body::empty())
        }
    } else {
        println!("Session is not found");
        builder.status(403).body(Body::empty())
    }
    .unwrap();

    Ok(res)
}

async fn response(req: Request<Body>, store: Arc<dyn Storable>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => home(req, store).await,
        (&Method::POST, "/session") => login(req, store).await,
        (&Method::POST, "/logout") => logout(req, store).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOTFOUND.into())
                .unwrap())
        }
    }
}

async fn serve(addr: SocketAddr, store: Arc<dyn Storable>) -> Result<()> {
    let new_service = make_service_fn(move |_| {
        let store = store.clone();
        async {
            Ok::<_, GenericError>(service_fn(move |mut req| {
                let cookie_data = CookieData::from_headers(req.headers(), http::header::COOKIE);
                req.extensions_mut().insert(cookie_data);
                response(req, store.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn respond(addr: SocketAddr, store: Arc<dyn Storable>) -> Result<()> {
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
    assert_eq!(session.status()?, SessionStatus::Existed);
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

#[test]
fn run() {
    let mut rt = Runtime::new().unwrap();

    let addr = "127.0.0.1:1337".parse().unwrap();

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
