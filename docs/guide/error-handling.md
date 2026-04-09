# Error Handling

> Result and Option types, the `?` operator, custom error types, `@error` derive, and error context.

## Philosophy

Sploosh has no `null`, no exceptions, no `throw`/`catch`. All fallible operations return `Result<T, E>` or `Option<T>`. This is enforced at the compiler level.

## The `?` Operator

Propagates errors early -- unwraps `Ok`/`Some` or returns `Err`/`None` from the enclosing function.

```sploosh
fn read_config(path: &str) -> Result<Config, FileError> {
    let content = fs::read(path)?;          // returns Err early if failed
    let parsed = json::parse(&content)?;
    Ok(Config::from(parsed))
}
```

## Custom Error Types

```sploosh
enum AppError {
    NotFound { resource: String },
    Unauthorized,
    Database(DbError),
    Network(NetError),
}

impl From<DbError> for AppError {
    fn from(e: DbError) -> Self { AppError::Database(e) }
}
```

## The `@error` Derive

Eliminates boilerplate for common error enum patterns:

```sploosh
@error
enum AppError {
    NotFound { resource: String },      // Display: "not found: {resource}"
    Unauthorized,                       // Display: "unauthorized"
    Database(DbError),                  // auto: impl From<DbError>
    Network(NetError),                  // auto: impl From<NetError>
}
```

**Rules:**
- Tuple variants (`Database(DbError)`) generate `From` impls
- Struct variants generate Display strings from field names
- Unit variants use snake_case name as display string
- Only one `From` impl per source type

## Error Context

```sploosh
let content = fs::read(path)
    .context(format("failed to read config from {}", path))?;
```

Wraps the original error, preserving the chain. Printed as: `"failed to read config from ./app.json: file not found"`.

## Option Type

```sploosh
fn find_user(id: UserId) -> Option<User> { /* ... */ }

let email = find_user(42)
    |> map(|u| u.email)
    |> unwrap_or("unknown@example.com".into());
```

## Next Steps

- [Closures and Iterators](closures-and-iterators.md)
- [Pipe Operator](pipe-operator.md)
