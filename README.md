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

### Storages

- [x] Memory
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
