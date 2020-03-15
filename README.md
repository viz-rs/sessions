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

- Async/await, supports [`tokio`](https://tokio.rs) and [`async-std`](https://async.rs/).
  **tokio** is by defaults.

- Stores the values in a [`Map<String, Value>`](https://docs.rs/serde_json/latest/serde_json/map/index.html) based on **serde_json**.

- Uses the [`nanoid!(32)`](https://docs.rs/nanoid) for generating `sid / UID`.

- Easy custom Stores.

### Examples

```rust
let store = Arc::new(CustomStore::new());

let id = "it is an unique ID.";                 // Generates an UID
let store = store.clone();
let session = store.get(&id).await;             // Fresh Session

session.id().await;                             // ""
session.status().await;                         // SessionStatus::Created
session.state().await;                          // State

session.set::<usize>("counter", 0).await;       // None
session.set("number", 233).await;               // None
session.get::<usize>("counter").await;          // Some(0)
session.get::<u32>("number").await;             // Some(233)

session.save().await;                           // bool

let session = store.get(&id).await;             // Matches Session

session.id().await;                             // "id.len() == 32"
session.status().await;                         // SessionStatus::Existed

session.remove::<usize>("counter").await;       // Some(0)
session.remove::<u32>("number").await;          // Some(233)

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    name: String,
    no: u32,
}

session.remove::<User>("user").await.is_some(); // true

session.set("user", User {
    name: "Yao Ming",
    no: 11,
}).await;                                       // None

session.get::<User>("user").await;              // Option<User>

session.destroy().await;                        // bool

session.status().await;                         // SessionStatus::Destroyed
// or

store.remove(&id).await;                        // bool
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
