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

## Spawning and Handles

```sploosh
let counter: Handle<Counter> = spawn Counter::init(0);
```

`Handle<T>` implements `Clone` and `Send`. Handles can be stored in structs, passed between actors, and put in collections.

## Message Passing

| Method type | Call style | Behavior |
|------------|-----------|----------|
| `&self` | Direct call: `counter.get()` | Request/reply, blocks caller |
| `&mut self` | `send counter.increment(5)` | Fire-and-forget, non-blocking |
| `&mut self` | Direct call: `counter.increment(5)` | Request/reply, blocks caller |

## Owned Parameters Only

All `pub` actor method parameters must be owned types. No `&T` or `&mut T` -- messages are async and the caller's stack frame may not exist when processed.

```sploosh
// CORRECT: owned String
pub fn log(&mut self, msg: String) { /* ... */ }

// COMPILE ERROR: &str is a reference
// pub fn log_ref(&mut self, msg: &str) { /* ... */ }
```

Private methods can use references freely.

## Generic Actors

```sploosh
actor Cache<K: Hash + Eq + Send, V: Clone + Send> {
    data: Map<K, V>,
    fn init() -> Self { Cache { data: Map::new() } }
    pub fn set(&mut self, key: K, value: V) { self.data.insert(key, value); }
    pub fn get(&self, key: &K) -> Option<V> { self.data.get(key).map(|v| v.clone()) }
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

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 3)
actor WorkerPool {
    children: Vec<Handle<Worker>>,
    fn init(size: u32) -> Self { /* spawn children */ }
}
```

## Actor Failure and Recovery

Actors die from runtime checks (bounds, overflow, assert). There is no `panic` keyword.

- Dead actor: request/reply returns `Err(ActorError::Dead)`
- `send` to dead actor: silently drops
- Supervised actors are restarted per strategy

```sploosh
match worker.process(data) {
    Ok(result) => use_result(result),
    Err(ActorError::Dead) => {
        let worker = spawn Worker::init();
        worker.process(data)?
    }
    Err(e) => return Err(AppError::from(e)),
}
```

## Next Steps

- [Async/Await](async-await.md)
- [Modules and Visibility](modules-and-visibility.md)
