#![cfg(all(feature = "redis", feature = "session"))]

use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::Result;
use redis::aio::ConnectionManager;

use sessions::*;

#[tokio::test]
async fn redis() -> Result<()> {
    let config = Arc::new(Store::new(
        RedisStorage::new(
            ConnectionManager::new(RedisClient::open("unix:///tmp/redis.sock")?).await?,
        ),
        nano_id::base64::<32>,
        |sid: &str| sid.len() == 32,
    ));

    let id = (config.generate)();

    assert!(id.len() == 32);
    assert!((config.verify)(&id));

    let session = Session::new(Data::new());
    assert!(session.status().load(Ordering::Acquire) == sessions::UNCHANGED);

    assert!(session
        .set::<String>("crate", "sessions".to_string())
        .is_ok());
    assert!(session.status().load(Ordering::Acquire) == sessions::CHANGED);

    assert_eq!(session.get("crate")?, Some("sessions".to_string()));

    assert_eq!(session.remove("crate"), Some("sessions".into()));
    assert!(session.status().load(Ordering::Acquire) == sessions::CHANGED);

    assert_eq!(session.remove_as::<String>("crate"), None);
    assert!(session.status().load(Ordering::Acquire) == sessions::CHANGED);

    assert_eq!(session.get::<String>("crate")?, None);

    session.clear();
    assert!(session.status().load(Ordering::Acquire) == sessions::CHANGED);
    session.clear();
    assert!(session.status().load(Ordering::Acquire) == sessions::CHANGED);

    let session = Session::new(Data::new());
    assert!(session.status().load(Ordering::Acquire) == sessions::UNCHANGED);

    session.renew();
    assert!(session.status().load(Ordering::Acquire) == sessions::RENEWED);

    session.purge();
    assert!(session.status().load(Ordering::Acquire) == sessions::PURGED);

    Ok(())
}
