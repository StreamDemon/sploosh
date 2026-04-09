# std::db

> Database connection pooling and query builder.

**Available targets:** native only

```sploosh
use std::db::Pool;

let db = Pool::connect("postgres://localhost/myapp")?;
let rows = db.query("SELECT * FROM users WHERE id = $1", &[id]).await?;
```

Not available inside `onchain` modules (compile-time error).

<!-- TODO: Document full API (connection pooling, query builder, transactions, migrations) once implemented -->
