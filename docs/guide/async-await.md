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

## Spawn Async (Non-Actor Tasks)

Use `spawn async` to run an async function as a standalone task that is not an actor. It returns a `JoinHandle<T>` that you can `.await` to get the result.

```sploosh
let handle: JoinHandle<Result<Response, NetError>> = spawn async fetch_data("https://example.com");

// Do other work...

let response = handle.await?;   // wait for the task to finish
```

Multiple tasks can run concurrently:

```sploosh
let h1 = spawn async fetch_data("https://api.one.com");
let h2 = spawn async fetch_data("https://api.two.com");

let (r1, r2) = (h1.await?, h2.await?);
```

## Async in Actors

`.await` is allowed inside actor message handlers. While an actor is awaiting, it stays **busy** -- it does not process other messages in its mailbox until the `.await` completes.

```sploosh
actor Fetcher {
    cache: Map<String, String>,

    fn init() -> Self { Fetcher { cache: Map::new() } }

    pub async fn get(&mut self, url: String) -> Result<String, NetError> {
        if let Some(cached) = self.cache.get(&url) {
            return Ok(cached.clone());
        }
        let body = net::get(&url).await?.text();   // actor is busy during this await
        self.cache.insert(url, body.clone());
        Ok(body)
    }
}
```

Keep awaits short to avoid starving the actor's mailbox. For long-running I/O, consider using `spawn async` and sending the result back via a message.

**Important:** `&mut` borrows cannot be held across `.await` points. The compiler will reject code that tries to keep a mutable reference alive over a suspend.

```sploosh
pub async fn bad_example(&mut self) {
    let entry = self.cache.get_mut("key");   // &mut borrow starts
    // COMPILE ERROR: &mut borrow held across .await
    // net::get("https://example.com").await?;
    // Use the entry, then await separately instead.
}
```

## Next Steps

- [Actors and Concurrency](actors-and-concurrency.md)
- [Modules and Visibility](modules-and-visibility.md)
