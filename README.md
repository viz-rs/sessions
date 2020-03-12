<h1 align="center">Sessions</h1>
<div align="center">
  <p><strong>Sessions module for web services.</strong></p>
  <p>Sessions provides memory and filesystem sessions and infrastructure for custom session backends.</p>
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

- Easy custom Stores

- Stores the values in a [`Map<String, Value>`](https://docs.rs/serde_json/latest/serde_json/map/index.html) based on _serde_json_

### Examples

```rust
let store = Arc::new(CustomStore::new());

let id = format!("id.{}", 0);                   // Generates an UID
let store = store.clone();
let session = store.get(&id).await.unwrap();    // Fresh Session

session.id().unwrap();                          // ""
session.status().unwrap();                      // SessionStatus::Created
session.state().unwrap();                       // State

session.set::<usize>("counter", 0).unwrap();    // None
session.set("number", 233).unwrap();            // None
session.get::<usize>("counter").unwrap();       // Some(0)
session.get::<u32>("number").unwrap();          // Some(233)

session.save().await;                           // Ok(())

let session = store.get(&id).await.unwrap();    // Matches Session

session.id().unwrap();                          // "id.len() == 32"
session.status().unwrap();                      // SessionStatus::Existed

session.remove::<usize>("counter").unwrap();    // Some(0)
session.remove::<u32>("number").unwrap();       // Some(233)

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    name: String,
    no: u32,
}

session.remove::<User>("user").is_ok();         // true

session.set("user", User {
    name: "Yao Ming",
    no: 11,
}).unwrap();                                    // None

session.get::<User>("user").unwrap();           // Option<User>

session.destroy().await;                        // Ok(())

session.status().unwrap();                      // SessionStatus::Destroyed
// or

store.remove(&id).await;                        // Ok(())
```

### Stores

- [x] Memory
- [x] Filesystem
- [x] Redis
- [ ] sled
- [ ] Memcached
- [ ] Mongodb
- [ ] PostgreSQL
- [ ] MySQL/MariaDB

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.
