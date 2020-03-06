## sessions

Sessions provides cookie and filesystem sessions and infrastructure for custom session backends.

WIP

### Features

### Stores

```rust
let store = Arc::new(CustomStore::new());

let sid = format!("sid.{}", 0);                 // Generate an unique ID
let store = store.clone();
let session = store.get(&sid).await.unwrap();   // Session

session.sid();                                  // sid.0
session.fresh();                                // true

session.set::<usize>("counter", 0).unwrap();    // None
session.set("number", 233).unwrap();            // None
session.get::<usize>("counter").unwrap();       // Some(0)
session.get::<u32>("number").unwrap();          // Some(233)

session.save().await;                           // Ok(())

let session = store.get(&sid).await.unwrap();   // Session

session.sid();                                  // sid.0
session.fresh();                                // false

session.remove::<usize>("counter").unwrap();    // Some(0)
session.remove::<u32>("number").unwrap();       // Some(233)

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    no: u32,
    name: String,
}

session.remove::<User>("user").is_ok();         // true

session.set("user", User {
    no: 11,
    name: "Yao ming"
}).unwrap();                                    // None

session.get::<User>("user").unwrap();           // Option<User>

store.remove(&sid).await;                       // Ok(())
```

- Memory

- Filesystem

- Redis

- Mongodb

- PostgreSQL

- MySQL/MariaDB
