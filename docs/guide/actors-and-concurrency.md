# Actors and Concurrency

> Sploosh's actor model: defining actors, spawning, message passing, handles, supervision, and failure recovery.

## Defining an Actor

```sploosh
actor Counter {
    state: i64,

    fn init(start: i64) -> Self {
        Counter { state: start }
    }

    pub fn increment(&mut self, amount: i64) {
        self.state = self.state + amount;
    }

    pub fn get(&self) -> i64 {
        self.state
    }
}
```

Actors process one message at a time. No data races by construction.

`init` is infallible — its signature returns `Self`, not `Result<Self, E>`, and it cannot be marked `async`.

## Actor Lifecycle

Every actor is always in one of three states:

- **INITIALIZING** -- `spawn` has returned a handle, but `init` is still running. Messages queue in the mailbox and are delivered once the actor is ready.
- **READY** -- `init` has returned. The actor is processing messages under the one-handler-at-a-time rule.
- **DEAD** -- The actor has terminated. Its state has been dropped and it will never process another message.

If `init` panics (bounds check, overflow, failed `assert`), the actor transitions `INITIALIZING → DEAD` without ever reaching `READY`. The handle returned by `spawn` may therefore be observationally dead on the very first call: request/reply returns `Err(ActorError::Dead)`, and `send` silently drops. If the actor is supervised, its parent is notified as if a `READY` child had died — init failures count toward `max_restarts`.

Need recoverable initialization? Store the work as an `Option<T>` field and run a handshake message after spawn:

```sploosh
actor Loader {
    config: Option<Config>,
    fn init() -> Self { Loader { config: None } }

    pub fn start(&mut self, path: String) -> Result<(), AppError> {
        self.config = Some(Config::load(&path)?);
        Ok(())
    }
}
```

## Spawning and Handles

```sploosh
let counter: Handle<Counter> = spawn Counter::init(0);
```

`Handle<T>` implements `Clone` and `Send`. Handles can be stored in structs, passed between actors, and put in collections.

**Handle drops do not kill the actor.** Unlike `Rc<T>` / `Arc<T>` (not available in Sploosh), `Handle<T>` is *not* reference-counted — dropping the last live handle has no effect on the actor's lifetime. Actors terminate only via runtime failure, supervisor termination, or runtime shutdown when `main()` returns. An actor with no live handle and an empty mailbox is *orphaned* and continues running until the runtime shuts down; clean shutdown is the supervisor's job. For shared immutable reference-counting (configs, lookup tables, read-only caches) use `Shared<T>` — see §4.4a of the language spec and the "Sharing Read-Heavy Data" section below.

## Message Passing

| Method type | Call style | Behavior |
|------------|-----------|----------|
| `&self` | Direct call: `counter.get()` | Request/reply, blocks caller |
| `&mut self` | `send counter.increment(5)` | Fire-and-forget, non-blocking |
| `&mut self` | Direct call: `counter.increment(5)` | Request/reply, blocks caller |

**`send` is only valid on `&mut self` methods.** Writing `send counter.get()` on a `&self` method is a compile error — discarding a pure query's reply is never the author's intent.

## Message Ownership Rules

The parameter-reference rule is keyed to the method's receiver:

- **`&mut self` methods** may be invoked via `send`, `send_timeout`, or direct request/reply. Because `send` can outlive the caller's stack frame, every parameter must be an owned type. `&T`, `&mut T`, and any type holding a non-`'static` reference are compile errors.
- **`&self` methods** are synchronous request/reply only — the caller always blocks until the reply arrives, so the caller's stack frame is guaranteed to outlive the call. `&self` methods **may** take reference parameters.
- **Private (non-`pub`) methods** are called only during message handling and may use references freely.

```sploosh
actor Logger {
    entries: Vec<String>,
    fn init() -> Self { Logger { entries: Vec::new() } }

    // CORRECT: &mut self with owned String
    pub fn log(&mut self, msg: String) { self.entries.push(msg); }

    // COMPILE ERROR: &mut self must use owned types
    // pub fn log_ref(&mut self, msg: &str) { /* ... */ }

    // OK: &self methods may take references
    pub fn count_matching(&self, needle: &str) -> u64 {
        self.entries.iter()
            |> filter(|e| e.contains(needle))
            |> count
    }
}
```

## Generic Actors

```sploosh
actor Cache<K: Hash + Eq + Send, V: Clone + Send> {
    data: Map<K, V>,
    fn init() -> Self { Cache { data: Map::new() } }
    pub fn set(&mut self, key: K, value: V) { self.data.insert(key, value); }
    // OK: &self request/reply methods may take references.
    pub fn get(&self, key: &K) -> Option<V> { self.data.get(key).map(|v| v.clone()) }
}
```

Every type parameter on an `actor` declaration must carry `Send`, not only the parameters used in `pub` method signatures. State fields may hold values of any of the parameters, and those fields move across scheduler threads when the actor is rebalanced between cores.

## Sharing Read-Heavy Data: `Shared<T>` Across Actors

When many actors need to read the same immutable value — a parsed configuration, a lookup table, cached ML weights, an interned-string pool — wrapping it in an actor forces every read through a message round-trip (mailbox enqueue plus scheduler context switch plus reply), which serializes what should be pointer-speed access. Deep-cloning into each actor wastes memory and allocates per spawn. `Shared<T>` (§4.4a of the language spec) is the idiomatic middle ground: an atomically refcounted, immutable-only pointer that lets every holder see the same allocation.

```sploosh
// Without Shared<T> — every lookup is a message round-trip.
actor LookupTableActor {
    table: LookupTable,
    fn init(table: LookupTable) -> Self { LookupTableActor { table } }
    pub fn get(&self, key: String) -> Option<u64> { self.table.get(&key) }
}

// With Shared<T> — workers hold a refcounted pointer and read directly.
actor Worker {
    table: Shared<LookupTable>,
    fn init(table: Shared<LookupTable>) -> Self { Worker { table } }
    pub fn lookup(&self, key: &str) -> Option<u64> {
        (*self.table).get(key)     // &T access across the refcounted pointer
    }
}

fn main() -> Result<(), AppError> {
    let table = Shared::new(LookupTable::load("data.bin")?);
    let w1 = spawn Worker::init(table.clone());
    let w2 = spawn Worker::init(table.clone());
    // Both workers share one allocation; each .clone() is an O(1) refcount bump.
    Ok(())
}
```

`Shared<T>` is `Clone + Send + Sync` iff `T: Send + Sync`. Each `.clone()` is a refcount bump (no allocation, no `T::clone` call). When the last `Shared<T>` clone drops, the inner `T` drops and the allocation is freed. Pick `Shared<T>` whenever the answer to *does any actor need to mutate this value?* is no; if yes, wrap in an actor and share the `Handle<T>`.

`Shared<T>` satisfies the §8.2 owned-parameter rule for `&mut self` actor methods (the wrapper itself moves; the inner data is shared via refcount bump) and is the idiomatic reply type for `&self` request/reply methods that return cached data — the caller bumps the refcount on receive rather than deep-cloning the reply.

## Channels

Channels provide typed, bounded communication between actors or async tasks.

```sploosh
let (tx, rx) = Channel::new(16);   // bounded channel with capacity 16

// Sender side
tx.send("hello".into());           // blocks if the channel is full

// Receiver side
let msg: String = rx.recv();       // blocks until a message is available
```

`Sender<T>` and `Receiver<T>` are the two halves of a channel. Both implement `Send` so they can be passed to other actors.

```sploosh
// With timeout (send_timeout is a compiler intrinsic)
match send_timeout(tx.send("hello".into()), 500) {
    Ok(()) => { /* sent */ }
    Err(SendError::Timeout) => { /* channel full after 500ms */ }
}
```

## Mailbox Configuration

Every actor has a bounded mailbox. Use the `@mailbox` attribute to configure its capacity. The default capacity is 1024. When the mailbox is full, the sender blocks until space is available.

```sploosh
@mailbox(capacity: 256)
actor Logger {
    state: Vec<String>,

    fn init() -> Self { Logger { state: Vec::new() } }

    pub fn log(&mut self, msg: String) {
        self.state.push(msg);
    }
}
```

## Select (Multiplexed Receive)

```sploosh
select {
    msg = recv channel_a => handle_a(msg),
    msg = recv channel_b => handle_b(msg),
    _ = timeout(5000) => return Err(AppError::Timeout),
}
```

## Supervision

Supervisors monitor child actors and apply a restart strategy when a child fails. Three strategies are available:

- **one_for_one** -- only the failed child is restarted.
- **one_for_all** -- all children are stopped and restarted when any child fails.
- **rest_for_one** -- the failed child and all children started after it are restarted.

Each strategy accepts `max_restarts` (maximum restart count) and `window_secs` (the time window in seconds for counting restarts). The window is **sliding** — each restart is tagged with its wall-clock timestamp and the supervisor counts restarts whose timestamps fall within the last `window_secs` seconds. If `max_restarts` is exceeded, the supervisor itself fails.

### Restart Semantics

When a supervisor restarts a child:

1. The failed actor's state is **dropped** (RAII runs any `Drop` impls). Its mailbox is discarded — queued messages are lost.
2. A **fresh `init`** is run with the arguments the supervisor originally used to spawn the child. The new instance begins in `INITIALIZING` and transitions to `READY` when `init` completes. State is *not* preserved across restart — this matches OTP semantics and avoids resurrecting a corrupted field into a state that caused the crash.
3. The supervisor's stored handle is replaced. **Any `Handle<T>` cloned out of the supervisor before the crash is permanently dead.** Callers that need to reach the restarted actor must re-fetch the handle from the supervisor — old handles are never transparently redirected.

Init failures count toward `max_restarts`: an actor that dies in `init` is reported to its supervisor exactly as if a `READY` child had died. This prevents infinite restart storms on a bad config.

`rest_for_one` requires children to be tracked in an ordered collection (e.g. `Vec<Handle<T>>`). A supervisor that spawns children dynamically into an unordered structure has no observable "started after" ordering — the compiler issues a warning and `rest_for_one` falls back to `one_for_one` at runtime.

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60)
actor WorkerPool {
    children: Vec<Handle<Worker>>,
    fn init(size: u32) -> Self { /* spawn children */ }
}

@supervisor(strategy: "one_for_all", max_restarts: 3, window_secs: 30)
actor Pipeline {
    reader: Handle<Reader>,
    processor: Handle<Processor>,
    writer: Handle<Writer>,
    fn init() -> Self { /* spawn children */ }
}

@supervisor(strategy: "rest_for_one", max_restarts: 4, window_secs: 60)
actor StagedPipeline {
    stages: Vec<Handle<Stage>>,
    fn init() -> Self { /* spawn ordered stages */ }
}
```

## Actor Failure and Recovery

Actors die from runtime checks (bounds, overflow, assert). There is no `panic` keyword.

- Dead actor: request/reply returns `Err(ActorError::Dead)`
- `send` to dead actor: silently drops
- Blocked sender on a full mailbox wakes immediately with `Err(SendError::Dead)` (for `send_timeout`) or silent drop (for `send`) if the destination dies. Supervisor restart does **not** transparently redirect blocked senders to the new instance.
- Supervised actors are restarted per strategy

```sploosh
match worker.process(data) {
    Ok(result) => use_result(result),
    Err(ActorError::Dead) => {
        let worker = spawn Worker::init();
        worker.process(data)?
    }
    Err(ActorError::SelfCall) => {
        // A handler called back into its own actor via request/reply — the runtime
        // catches this direct self-deadlock and returns SelfCall instead of hanging.
        return Err(AppError::Logic("self-call"));
    }
    Err(e) => return Err(AppError::from(e)),
}
```

## Re-entrant Calls and Deadlock

An actor holds its mailbox "busy" across a handler's entire execution, including every `.await` point. A request/reply call from a handler back into the same actor therefore deadlocks — the caller is waiting for itself to return before it will process the next message.

**Direct self-calls** (A's handler calls request/reply on `Handle<A>`) are detected by the runtime and return `Err(ActorError::SelfCall)` immediately — no hang. This catches the common accident of writing `self.handle.method(args)` where a local `self.method(args)` was intended.

**Multi-actor cycles** (A → B → A, or longer chains) are **not detected** by the current runtime. Such chains block until an outer `send_timeout` or user-level timeout fires. Structure actor communication as a DAG or break cycles with fire-and-forget `send`.

**Self-sends are legal.** A handler that wants to re-queue work into its own mailbox should use `send self.handle.method(args)` — the message is processed on the next handler turn after the current one returns. This is the correct pattern for self-scheduling and for splitting long computations.

## Where Actors Can't Run

Actors are an off-chain primitive. The `actor` keyword, the `spawn`, `send`, `send_timeout`, `select`, and `timeout(ms)` intrinsics, and the `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, and `JoinHandle<T>` types are all compile errors inside `onchain` modules. The `@supervisor` and `@mailbox` attributes are also rejected on items in `onchain` scope. On-chain execution is synchronous, single-threaded, and transactional — there is no runtime scheduler for any of this to run on. See the [on-chain overview](../web3/onchain-overview.md) for details.

## Next Steps

- [Async/Await](async-await.md)
- [Modules and Visibility](modules-and-visibility.md)
