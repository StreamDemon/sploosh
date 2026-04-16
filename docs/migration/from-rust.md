# Coming from Rust

> What's the same, what's different, and what to watch for.

## What's the Same
- Ownership and borrowing model (single owner, `&T`/`&mut T`)
- `Result<T, E>` and `Option<T>` error handling with `?`
- Pattern matching with `match` (exhaustive)
- `struct`, `enum`, `trait`, `impl`
- Generics with trait bounds
- `let`/`const` bindings
- `Vec<T>`, `Map<K,V>`, `Set<T>`

## Key Differences

| Rust | Sploosh | Notes |
|------|---------|-------|
| `format!("...", x)` | `format("...", x)` | Compiler intrinsic, not a macro |
| `#[derive(...)]` | `@derive(...)` | `@` instead of `#[]` for attributes |
| `#[test]` | `@test` | Same semantics, different syntax |
| Lifetime elision (3 rules) | Minimal elision (single-source rule) | Only elides when there is exactly one input lifetime |
| `pub(crate)`, `pub(super)` | Just `pub` | Two visibility levels only |
| `static mut` | Not available | All mutable state lives in actors |
| Threads + `Arc<Mutex<T>>` | `Shared<T>` for reads; actors + `Handle<T>` for writes | `Shared<T>` replaces `Arc<T>` for read-only sharing; actors replace `Arc<Mutex<T>>` when mutation is needed |
| `tokio::spawn` / `async-std` | `spawn` (actors), `async`/`await` (I/O) | Built-in actor runtime |
| `+` for String concat | Not available | Use `format()` or `push_str` |
| `panic!()` | Not available | Actors die from runtime checks only |
| Macros (`macro_rules!`) | Not available | No macro system |
| `HashMap`, `HashSet` | `Map`, `Set` | Shorter names |
| `println!("...")` | `print("...")` | Not a macro |
| `x as u32` | `x as u32` | Numeric casts work the same way |
| `unsafe { ... }` | Not available | Use `extern "C"` with safe wrappers instead |
| `Rc<T>` / `Arc<T>` | `Shared<T>` (immutable) or `Handle<T>` (actor) | `Shared<T>` is strictly less than `Arc<T>` — immutable-only, no `Weak`, no cycles. Reach for `Handle<T>` when mutation is needed. |
| Unbounded `mpsc::channel` | `Channel::new(cap)` | Bounded MPSC only; returns `(Sender<T>, Receiver<T>)` |
| Unbounded actor channels | Bounded actor mailboxes | Mailboxes have a fixed capacity with backpressure |
| Checked arithmetic (debug only) | Checked arithmetic (all modes) | Overflow always panics, no silent wrapping in release |
| `tokio::task::spawn_blocking` | Not available yet | The only way to run blocking work from an actor handler is an `extern "C" async` FFI wrapper, which the compiler offloads to the runtime's blocking pool. Calling plain `extern "C"` from a handler is a compile error. |
| `tokio::sync::Mutex` held across `.await` | Not needed | The actor holds its `&mut self` lock automatically until the handler returns (including across `.await`). Direct re-entrant self-calls are detected as `ActorError::SelfCall` rather than deadlocking. |
| `Arc::strong_count` / dropping the last `Arc` frees the inner value | `Handle<T>` is not reference-counted; `Shared<T>` is | Dropping the last `Handle<T>` does not kill the actor — terminate via supervisor or runtime shutdown; orphaned actors keep running. `Shared<T>` *is* refcounted and is deterministically freed at count zero, matching `Arc<T>` drop semantics. |

## New Concepts for Rust Developers
- **Actors** (`actor`, `spawn`, `send`, `Handle<T>`) -- structured concurrency primitive
- **Pipe operator** (`|>`) -- data transformation chains
- **`onchain` modules** -- smart contract code with compile-time stdlib restrictions
- **`@error` derive** -- auto-generates `From`, `Display`, `Error` for error enums
