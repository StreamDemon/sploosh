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
| Lifetime elision | No elision | All lifetimes must be explicit |
| `pub(crate)`, `pub(super)` | Just `pub` | Two visibility levels only |
| `static mut` | Not available | All mutable state lives in actors |
| Threads + `Arc<Mutex<T>>` | Actors + `Handle<T>` | Actor model instead of shared-memory concurrency |
| `tokio::spawn` / `async-std` | `spawn` (actors), `async`/`await` (I/O) | Built-in actor runtime |
| `+` for String concat | Not available | Use `format()` or `push_str` |
| `panic!()` | Not available | Actors die from runtime checks only |
| Macros (`macro_rules!`) | Not available | No macro system |
| `HashMap`, `HashSet` | `Map`, `Set` | Shorter names |
| `println!("...")` | `print("...")` | Not a macro |

## New Concepts for Rust Developers
- **Actors** (`actor`, `spawn`, `send`, `Handle<T>`) -- structured concurrency primitive
- **Pipe operator** (`|>`) -- data transformation chains
- **`onchain` modules** -- smart contract code with compile-time stdlib restrictions
- **`@error` derive** -- auto-generates `From`, `Display`, `Error` for error enums
