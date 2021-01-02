<h1 align="center">Sessions</h1>
<div align="center">
  <p><strong>General sessions module for web services.</strong></p>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/sessions">
    <img src="https://img.shields.io/crates/v/sessions.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/sessions">
    <img src="https://img.shields.io/crates/d/sessions.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/sessions">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

### Features

- Async/await

- Easy custom Storages

- Stores the values in a [`Map<String, Value>`](https://docs.rs/serde_json/latest/serde_json/map/index.html) based on _serde_json_

### Example

```toml
sessions = { version = "0.1", features = ["memory"] }
```

```rust
use std::sync::Arc;
use sessions::*;

let config = Arc::new(Config {
  cookie: CookieOptions::new(),
  storage: Arc::new(MemoryStorage::default()),
  //storage: Arc::new(RedisStorage::new(RedisClient::open("redis://127.0.0.1")?)),
  generate: Box::new(|| nanoid::nanoid!(32)),
  verify: Box::new(|sid: &str| sid.len() == 32),
});


let session = Session::new(&config.generate(), 1, config.clone());
session.set::<String>("crate", "sessions".to_string());
let val: Option<String> = session.get("crate");
session.remove("crate");
session.clear();

session.save().await;
session.renew().await;
session.destroy().await;
```

### Storages

- [x] Memory
- [x] Redis
- [ ] sled
- [ ] Memcached
- [ ] Mongodb
- [ ] PostgreSQL
- [ ] MySQL/MariaDB

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
