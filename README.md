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

session.set::<usize>("counter", i).unwrap();    // None
session.set("number", 233).unwrap();            // None
session.get::<usize>("counter").unwrap();       // Some(i)
session.get::<u32>("number").unwrap();          // Some(233)

session.save().await;                           // Ok(())

store.remove(&sid).await;                       // Ok(())
```

- MemoryStore

- FilesystemStore

- RedisStore

- MongodbStore

- PostgresStore

- MySQL/MariaDB Store
