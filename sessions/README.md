<h1 align="center">Sessions</h1>

<div align="center">
  <p><strong>General sessions module for web services</strong></p>
</div>

<br />

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square"
      alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/sessions">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/sessions">
    <img src="https://img.shields.io/crates/v/sessions.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/sessions">
    <img src="https://img.shields.io/crates/d/sessions.svg?style=flat-square"
      alt="Download" /></a>
  <!-- Discord -->
  <a href="https://discord.gg/cjX2KX">
     <img src="https://img.shields.io/discord/699908392105541722?logo=discord&style=flat-square"
     alt="Discord"></a>
  <!-- Twitter -->
  <a href="https://twitter.com/_fundon">
    <img src="https://img.shields.io/badge/twitter-@__fundon-blue.svg?style=flat-square"
      alt="Twitter: @_fundon" /></a>
</div>

### Features

- Async/await

- Easy custom Storages

- Stores the values in a `BTreeMap<String, Value>`

### Example

```toml
sessions = { version = "0.4", features = ["memory"] }
```

```rust
use std::sync::Arc;
use sessions::*;

let config = Arc::new(Store::new(
  MemoryStorage::new(),
  nano_id::base64::<32>,
  |sid: &str| sid.len() == 32,
));


let session = Session::new(Data::new());
session.set::<String>("crate", "sessions".to_string());
let val: Option<String> = session.get("crate");
session.remove("crate");
session.clear();
```

### Storages

- [x] Memory
- [x] Redis
- [ ] SQLx
  - [ ] SQLite
  - [ ] PostgreSQL
  - [ ] MySQL/MariaDB
- [ ] Memcached
- [ ] Mongodb

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
