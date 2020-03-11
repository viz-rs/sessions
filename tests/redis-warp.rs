// use cookie::{Cookie, CookieJar};
// use redis::Client as RedisClient;
// use serde::{Deserialize, Serialize};
// use serde_json::{from_slice, to_value};
// use sessions::{RedisStore, Session, SessionStatus, Storable};
// use std::{
//     io::Error,
//     net::SocketAddr,
//     sync::{Arc, RwLock},
// };
// use time::Duration;
// use tokio::runtime::Runtime;
// use warp::{
//     http::{header, Response},
//     hyper::{Body, Client, HeaderMap, Method, Request},
//     reject::{not_found, Reject},
//     Filter, Rejection, Reply,
// };

// static SESSION_NAME: &str = "session.id";

// #[derive(Debug, Serialize, Deserialize, PartialEq)]
// struct User {
//     id: String,
//     logged_in: bool,
//     count: u32,
// }

// #[derive(Debug)]
// struct MyError;

// impl Reject for MyError {}

// type GenericError = Box<dyn std::error::Error + Send + Sync>;
// type GenericResult<T> = std::result::Result<T, GenericError>;

// // From https://github.com/http-rs/tide/blob/master/src/middleware/cookies.rs
// #[derive(Debug, Clone)]
// struct CookieData {
//     content: Arc<RwLock<CookieJar>>,
// }

// impl CookieData {
//     fn from_headers(headers: &HeaderMap, key: header::HeaderName) -> Self {
//         let cookie_header = headers.get(key);
//         let cookie_jar = cookie_header.and_then(|raw| {
//             let mut jar = CookieJar::new();

//             // as long as we have an ascii string this will start parsing the cookie
//             if let Some(raw_str) = raw.to_str().ok() {
//                 raw_str
//                     .split(';')
//                     .try_for_each(|s| -> GenericResult<_> {
//                         jar.add_original(Cookie::parse(s.trim().to_owned())?);
//                         Ok(())
//                     })
//                     .ok()?;
//             }

//             Some(jar)
//         });
//         let content = Arc::new(RwLock::new(cookie_jar.unwrap_or_default()));

//         CookieData { content }
//     }
// }

// async fn respond(addr: SocketAddr, store: Arc<dyn Storable>) -> GenericResult<()> {
//     // First visit home.
//     let req = Request::builder()
//         .uri(format!("http://{}/", addr))
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 200);
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(buf.is_empty());

//     // First Login.
//     let req = Request::builder()
//         .uri(format!("http://{}/{}", addr, "session"))
//         .method(Method::POST)
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 200);
//     let cookie_data = CookieData::from_headers(res.headers(), header::SET_COOKIE);
//     let content = cookie_data.content.read().unwrap();
//     let id = content.get(SESSION_NAME).map(|c| c.value());
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(!buf.is_empty());
//     let user = from_slice::<User>(&buf)?;
//     assert_eq!(id, Some(user.id.as_str()));
//     assert_eq!(0, user.count);

//     let session = store.get(id.unwrap()).await?;
//     assert_eq!(session.status()?, SessionStatus::Created);
//     assert_eq!(
//         to_value(user)?.as_object().map(|m| m.to_owned()).unwrap(),
//         session.state()?
//     );

//     // Second Login.
//     let req = Request::builder()
//         .uri(format!("http://{}/{}", addr, "session"))
//         .header(
//             header::COOKIE,
//             cookie_data
//                 .content
//                 .read()
//                 .unwrap()
//                 .get(SESSION_NAME)
//                 .unwrap()
//                 .encoded()
//                 .to_string(),
//         )
//         .method(Method::POST)
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 200);
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(!buf.is_empty());
//     let user = from_slice::<User>(&buf)?;
//     assert_eq!(id, Some(user.id.as_str()));
//     assert_eq!(1, user.count);

//     let session = store.get(id.unwrap()).await?;
//     assert_eq!(session.status()?, SessionStatus::Existed);
//     assert_eq!(
//         to_value(user)?.as_object().map(|m| m.to_owned()).unwrap(),
//         session.state()?
//     );

//     // Second visit home.
//     let req = Request::builder()
//         .uri(format!("http://{}/", addr))
//         .header(
//             header::COOKIE,
//             cookie_data
//                 .content
//                 .read()
//                 .unwrap()
//                 .get(SESSION_NAME)
//                 .unwrap()
//                 .encoded()
//                 .to_string(),
//         )
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 200);
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(!buf.is_empty());
//     let user = from_slice::<User>(&buf)?;
//     assert_eq!(id, Some(user.id.as_str()));
//     assert_eq!(2, user.count);

//     let session = store.get(id.unwrap()).await?;
//     assert_eq!(session.status()?, SessionStatus::Existed);
//     assert_eq!(
//         to_value(user)?.as_object().map(|m| m.to_owned()).unwrap(),
//         session.state()?
//     );

//     // First logout.
//     let req = Request::builder()
//         .uri(format!("http://{}/{}", addr, "logout"))
//         .header(
//             header::COOKIE,
//             cookie_data
//                 .content
//                 .read()
//                 .unwrap()
//                 .get(SESSION_NAME)
//                 .unwrap()
//                 .encoded()
//                 .to_string(),
//         )
//         .method(Method::POST)
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 200);
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(!buf.is_empty());
//     let user = from_slice::<User>(&buf)?;
//     assert_eq!(id, Some(user.id.as_str()));
//     assert_eq!(3, user.count);

//     let session = store.get(id.unwrap()).await?;
//     assert_eq!(session.status()?, SessionStatus::Created);
//     assert!(session.state()?.is_empty());

//     // Second logout.
//     let req = Request::builder()
//         .uri(format!("http://{}/{}", addr, "logout"))
//         .method(Method::POST)
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 403);
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(buf.is_empty());

//     // Three visit home.
//     let req = Request::builder()
//         .uri(format!("http://{}/", addr))
//         .body(Body::empty())
//         .unwrap();
//     let res = Client::new().request(req).await?;
//     assert_eq!(res.status(), 200);
//     let buf = hyper::body::to_bytes(res).await?;
//     assert!(buf.is_empty());

//     dbg!(id.unwrap());
//     Ok(())
// }

// fn with_session(
//     store: Arc<dyn Storable>,
// ) -> impl Filter<Extract = (Session,), Error = Rejection> + Clone {
//     warp::any()
//         .map(move || store.clone())
//         .and(warp::filters::cookie::optional(SESSION_NAME))
//         .and_then(
//             move |store: Arc<dyn Storable>, cookie: Option<String>| async move {
//                 let sid = cookie
//                     .filter(|id| id.len() == 32)
//                     .unwrap_or_else(|| "".to_owned());
//                 match store.get(&sid).await {
//                     Ok(session) => Ok(session),
//                     Err(err) => {
//                         dbg!(err);
//                         Err(not_found())
//                     }
//                 }
//             },
//         )
// }

// async fn home(session: Session) -> Result<impl Reply, Rejection> {
//     async fn run(session: Session) -> Result<impl Reply, Error> {
//         let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

//         let mut count = session.get::<usize>("count")?.unwrap_or_else(|| 0);

//         Ok(if session.status()? == SessionStatus::Existed {
//             count += 1;
//             session.set("count", count)?;
//             session.save().await?;
//             println!("User is logged in, {}.", count);
//             builder.body(Body::from(serde_json::to_vec(&session.state()?)?))
//         } else {
//             println!("User is not logged in.");
//             builder.body(Body::empty())
//         }
//         .unwrap())
//     }

//     run(session)
//         .await
//         .map_err(|_| warp::reject::custom(MyError))
// }

// async fn login(session: Session) -> Result<impl Reply, Rejection> {
//     async fn run(session: Session) -> Result<impl Reply, Error> {
//         let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

//         let mut count = session.get::<usize>("count")?.unwrap_or_else(|| 0);

//         Ok(if session.status()? == SessionStatus::Existed {
//             count += 1;
//             session.set("count", count)?;
//             session.save().await?;
//             println!("User is logged in, {}.", count);
//             builder
//         } else {
//             session.set("logged_in", true)?;
//             session.set("count", count)?;
//             session.save().await?;
//             println!("User is logged in, {}.", count);
//             builder.header(
//                 header::SET_COOKIE,
//                 Cookie::new(SESSION_NAME, session.id()?)
//                     .encoded()
//                     .to_string(),
//             )
//         }
//         .body(Body::from(serde_json::to_vec(&session.state()?)?))
//         .unwrap())
//     }

//     run(session)
//         .await
//         .map_err(|_| warp::reject::custom(MyError))
// }

// async fn logout(session: Session) -> Result<impl Reply, Rejection> {
//     async fn run(session: Session) -> Result<impl Reply, Error> {
//         let builder = Response::builder().header(header::CONTENT_TYPE, "application/json");

//         let mut count = session.get::<usize>("count")?.unwrap_or_else(|| 0);

//         Ok(if session.status()? == SessionStatus::Existed {
//             count += 1;
//             println!("User is logged in, {}", count);
//             session.set("count", count)?;
//             println!("Session is destroyed");
//             session.destroy().await?;
//             let cookie = Cookie::build(SESSION_NAME, session.id()?)
//                 .max_age(Duration::seconds(-1))
//                 .finish();
//             builder
//                 .header(header::SET_COOKIE, cookie.encoded().to_string())
//                 .body(Body::from(serde_json::to_vec(&session.state()?)?))
//         } else {
//             println!("Session is not found");
//             builder.status(403).body(Body::empty())
//         }
//         .unwrap())
//     }

//     run(session)
//         .await
//         .map_err(|_| warp::reject::custom(MyError))
// }

// #[test]
// fn warp_with_memory() {
//     pretty_env_logger::init();

//     let mut rt = Runtime::new().unwrap();

//     let addr: std::net::SocketAddr = "127.0.0.1:1337".parse().unwrap();

//     let client = RedisClient::open("redis://127.0.0.1/").unwrap();

//     let store = RedisStore::new(client, "session:id:", 5);

//     let arc_store = Arc::new(store);

//     let store_1 = arc_store.clone();

//     // GET `/`
//     let home_route = warp::path::end()
//         .and(warp::get())
//         .and(with_session(arc_store.clone()))
//         .and_then(home);

//     // POST `/session`
//     let login_route = warp::path!("session")
//         .and(warp::post())
//         .and(with_session(arc_store.clone()))
//         .and_then(login);

//     // POST `/logout`
//     let logout_route = warp::path!("logout")
//         .and(warp::post())
//         .and(with_session(arc_store.clone()))
//         .and_then(logout);

//     let routes = home_route.or(login_route).or(logout_route);

//     rt.spawn(async move {
//         warp::serve(routes).run(addr).await;
//     });

//     rt.block_on(async move {
//         for _ in 0..10 {
//             let _ = respond(addr, store_1.clone()).await;
//         }
//     });
// }
