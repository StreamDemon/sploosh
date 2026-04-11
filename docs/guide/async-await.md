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

## Re-entrant Self-Calls

Because the mailbox stays locked for the entire handler (including every `.await` point), a synchronous request/reply call from a handler back into the same actor would deadlock. The runtime detects the direct case and returns `Err(ActorError::SelfCall)` immediately instead of hanging:

```sploosh
pub async fn process(&mut self, job: Job) -> Result<Report, ActorError> {
    // WRONG: request/reply on own handle → ActorError::SelfCall.
    // let count = self.self_handle.as_ref().unwrap().job_count()?;

    // CORRECT: self-schedule via fire-and-forget, which enqueues to the
    // actor's own mailbox and runs on the next handler turn.
    send self.self_handle.as_ref().unwrap().retry(job.clone());
    Ok(Report::queued())
}
```

Multi-actor cycles (A awaits B, B awaits A) are not detected — keep actor communication a DAG, or break cycles with fire-and-forget `send`.

## When Async Is Not Allowed

- **`init` cannot be `async`.** Actor `init` returns `Self`, not `Result<Self, E>`, and it cannot be marked `async`. Writing `async fn init(...)` on an actor is a compile error. Model recoverable initialization with an `Option<T>` field and a handshake message (see the [actors guide](actors-and-concurrency.md#actor-lifecycle)).
- **Blocking FFI in handlers is forbidden.** Actor message handlers run in an async context. Calling a synchronous `extern "C"` function from a handler (directly or transitively) is a compile error. FFI that needs to be handler-safe must be declared `extern "C" async`; the compiler emits an awaitable wrapper that offloads the underlying call to the runtime's blocking pool.
- **No actors in `onchain`.** `spawn`, `send`, `async`/`.await`, `Channel<T>`, `Handle<T>`, and every related construct are compile errors inside `onchain` modules — on-chain execution is synchronous and single-threaded within a transaction.

## Next Steps

- [Actors and Concurrency](actors-and-concurrency.md)
- [Modules and Visibility](modules-and-visibility.md)
