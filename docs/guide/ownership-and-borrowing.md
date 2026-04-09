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
        fs::remove(self.path);
    }
}
```

`Box<T>`, file handles, network connections, and other resource-owning types use `Drop` for deterministic cleanup. Drop is called automatically -- you never call it directly.

## No Rc/Arc -- Use Handle\<T\> for Sharing

Sploosh does not provide `Rc<T>` or `Arc<T>`. Shared mutable state belongs in actors; use `Handle<T>` to share access across actors. `Handle<T>` is `Clone + Send` and routes messages to the owning actor, so no reference counting or locking is needed.

```sploosh
let counter: Handle<Counter> = spawn Counter::init(0);
let h2 = counter.clone();     // cheap clone -- both handles talk to the same actor
```

## No Static Mutable State

There is no `static` keyword. All mutable state lives in actors. This eliminates data races by construction.

## Next Steps

- [Structs, Enums, and Traits](structs-enums-and-traits.md)
- [Closures and Iterators](closures-and-iterators.md)
