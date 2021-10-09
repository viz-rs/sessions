#![cfg(feature = "redis")]

use std::sync::Arc;

use anyhow::Result;

use sessions::*;

#[tokio::test]
async fn redis() -> Result<()> {
    let config = Arc::new(Config {
        cookie: CookieOptions::new(),
        storage: RedisStorage::new(RedisClient::open("unix:///tmp/redis.sock")?),
        generate: Box::new(nano_id::base64::<32>),
        verify: Box::new(|sid| sid.len() == 32),
    });

    let id = config.generate();

    let session = Session::new(&id, 0, config.clone());

    assert_eq!(session.set::<String>("crate", "sessions".to_string()), None);

    assert!(session.save().await.is_ok());

    assert_eq!(session.get("crate"), Some("sessions".to_string()));

    assert_eq!(
        session.remove::<String>("crate"),
        Some("sessions".to_string())
    );

    assert_eq!(session.remove::<String>("crate"), None);

    assert_eq!(session.get::<String>("crate"), None);

    assert!(session.clear().is_ok());

    let mut session = Session::new(&id, 0, config.clone());

    if let Some(data) = config.get(&id).await? {
        session.set_data(data)?;
    }

    assert_eq!(session.get("crate"), Some("sessions".to_string()));

    assert!(session.renew().await.is_ok());

    assert_ne!(id, session.id()?);

    assert!(session.destroy().await.is_ok());

    Ok(())
}
