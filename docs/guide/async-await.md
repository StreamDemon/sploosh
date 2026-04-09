# Async/Await

> Non-actor asynchronous programming for I/O-bound operations.

## Basic Async Functions

```sploosh
async fn fetch_data(url: &str) -> Result<Response, NetError> {
    let conn = net::connect(url).await?;
    let response = conn.get("/api/data").await?;
    Ok(response)
}
```

## When to Use Async vs Actors

- **Actors** -- for stateful, concurrent entities that process messages sequentially
- **Async/await** -- for I/O-bound operations (network requests, file reads) where you need to await external results

## Async with Error Propagation

`?` works naturally with `.await`:

```sploosh
async fn fetch(url: &str) -> Result<Response, NetError> {
    let r = net::get(url).await?;
    Ok(r)
}
```

<!-- TODO: Expand with runtime details, task spawning, and interaction with actor system once implemented -->

## Next Steps

- [Actors and Concurrency](actors-and-concurrency.md)
- [Modules and Visibility](modules-and-visibility.md)
