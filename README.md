## sessions

Sessions provides cookie and filesystem sessions and infrastructure for custom session backends.

WIP

### Features

### Stores

```rust
let store = Arc::new(CustomStore::new());

let id = format!("id.{}", 0);                 	// Generates an UID
let store = store.clone();
let session = store.get(&id).await.unwrap();   	// Session

session.id();                                  	// "id.0"
session.fresh();                                // true

session.set::<usize>("counter", 0).unwrap();    // None
session.set("number", 233).unwrap();            // None
session.get::<usize>("counter").unwrap();       // Some(0)
session.get::<u32>("number").unwrap();          // Some(233)

session.save().await;                           // Ok(())

let session = store.get(&id).await.unwrap();   	// Session

session.id();                                  	// "id.0"
session.fresh();                                // false

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

store.remove(&id).await;                       	// Ok(())
```

- Memory

- Filesystem

- Redis

- Mongodb

- PostgreSQL

- MySQL/MariaDB
