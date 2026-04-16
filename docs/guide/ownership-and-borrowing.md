# Ownership and Borrowing

> Sploosh's memory safety model: single ownership, move semantics, borrowing, and lifetimes.

This is the most important concept to understand in Sploosh. If you're coming from garbage-collected languages, this will be new. If you're coming from Rust, this will be familiar -- but note that Sploosh uses only minimal lifetime elision (see below).

## The Four Rules

1. Every value has exactly one owner.
2. When the owner goes out of scope, the value is dropped.
3. You may have either ONE `&mut` reference OR any number of `&` references. Never both.
4. References must always be valid (no dangling pointers).

## Move vs Copy

```sploosh
let a = String::from("hello");
let b = a;          // a is MOVED to b. a is no longer valid.

let x: i32 = 42;
let y = x;          // x is COPIED. Both valid. (primitives implement Copy)
```

Primitives (`i32`, `f64`, `bool`, `char`, etc.) implement `Copy` and are duplicated on assignment. All other types are moved.

## Borrowing

```sploosh
fn greet(name: &str) -> String {       // immutable borrow
    format("Hello, {}", name)
}

fn update_name(user: &mut User, new_name: String) {  // mutable borrow
    user.name = new_name;
}
```

## Box\<T\> (Heap Allocation)

`Box<T>` is a heap-allocated, single-owner smart pointer. When the `Box` goes out of scope, the heap memory is freed.

```sploosh
let b = Box::new(42);          // allocate an i64 on the heap
let val = *b;                  // dereference to get the inner value

let s = Box::new(String::from("hello"));
println("{}", *s);             // auto-deref also works for method calls
```

`Box<T>` implements the `Drop` trait, so cleanup happens automatically when it leaves scope. Use `Box<T>` when you need:

- A value with a known size on the heap (e.g., recursive types).
- Single ownership with deterministic deallocation.
- To move large data without copying.

## Lifetimes

Sploosh uses a **minimal elision rule** to reduce annotation noise in the common case:

- **Single-source rule:** when a function has exactly one reference input and one reference output, they share the same lifetime -- no annotation needed.
- **Multiple sources:** when more than one reference input could be the source of the returned reference, explicit lifetime annotations are required.

```sploosh
// Single source -- elided (one &str in, one &str out, same lifetime implied)
fn trim(s: &str) -> &str {
    s.trim()
}

// Multiple sources -- explicit annotation required
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

## Pattern Binding Rules

- Primitives are copied into pattern bindings.
- Non-Copy types are moved into pattern bindings.
- Use `ref` to borrow instead of move: `Some(ref name) => name.len()`.
- When matching on `&self` or `&T`, bindings are automatically references.

## Drop Trait (RAII Cleanup)

Types that need cleanup when they go out of scope implement the `Drop` trait. This is Sploosh's mechanism for RAII (Resource Acquisition Is Initialization).

```sploosh
struct TempFile {
    path: String,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove(&self.path);
    }
}
```

`Box<T>`, file handles, network connections, and other resource-owning types use `Drop` for deterministic cleanup. Drop is called automatically -- you never call it directly.

## Shared State: `Shared<T>` for Immutable, `Handle<T>` for Mutable

Sploosh does not provide `Rc<T>` or `Arc<T>`. Sharing is split by intent into two narrow primitives, each with one job:

- **`Shared<T>`** — an atomically refcounted, immutable-only pointer. `Clone + Send + Sync` when `T: Send + Sync`. Use it for configs, lookup tables, interned strings, ML weights, and any other value that is constructed once and read by many actors. Cloning bumps an atomic refcount in O(1) — no allocation, no `T::clone` call. When the last `Shared<T>` clone is dropped, `T` is dropped and the allocation is freed. There is no `&mut *shared`, no `get_mut`, no `Weak<T>` — `Shared<T>` can only ever produce `&T`, which is what makes reference cycles impossible and keeps the surface narrow. See §4.4a of the language spec.
- **`Handle<T>`** — a typed actor handle for shared mutable state. `Clone + Send` and routes messages to the owning actor, so no reference counting or locking is needed. See §8.2 of the language spec.

```sploosh
// Read-only data: cheap to clone into many actors.
let config: Shared<Config> = Shared::new(Config::load("app.toml")?);
let worker_a = spawn Worker::init(config.clone());
let worker_b = spawn Worker::init(config.clone());

// Mutable state: one actor owns, handles share access.
let counter: Handle<Counter> = spawn Counter::init(0);
let h2 = counter.clone();     // cheap clone -- both handles talk to the same actor
```

Pick by intent: *does any actor need to mutate this value?* If no, reach for `Shared<T>`; if yes, wrap it in an actor and share its `Handle<T>`.

**Neither primitive is available inside `onchain` modules.** Reference counting has no gas or storage meaning, and actors do not exist on-chain — see §11.1 and §12.3 of the language spec.

**`Handle<T>` is not reference-counted.** Dropping the last live handle has no effect on the actor's lifetime — actors die only via runtime failure, supervisor termination, or runtime shutdown. An actor with no live handle and an empty mailbox is *orphaned* and continues running until the runtime shuts down. Clean shutdown is the supervisor's job. `Shared<T>` *is* reference-counted and is deterministically freed when the last clone is dropped.

## No Static Mutable State

There is no `static` keyword. All mutable state lives in actors. This eliminates data races by construction.

## Next Steps

- [Structs, Enums, and Traits](structs-enums-and-traits.md)
- [Closures and Iterators](closures-and-iterators.md)
