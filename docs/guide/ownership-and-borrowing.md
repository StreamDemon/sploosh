# Ownership and Borrowing

> Sploosh's memory safety model: single ownership, move semantics, borrowing, and lifetimes.

This is the most important concept to understand in Sploosh. If you're coming from garbage-collected languages, this will be new. If you're coming from Rust, this will be familiar -- but note that Sploosh requires explicit lifetimes (no elision).

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

## Explicit Lifetimes

Sploosh does not elide lifetimes. When a function returns a reference, lifetimes must be annotated.

```sploosh
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

## Pattern Binding Rules

- Primitives are copied into pattern bindings.
- Non-Copy types are moved into pattern bindings.
- Use `ref` to borrow instead of move: `Some(ref name) => name.len()`.
- When matching on `&self` or `&T`, bindings are automatically references.

## No Static Mutable State

There is no `static` keyword. All mutable state lives in actors. This eliminates data races by construction.

## Next Steps

- [Structs, Enums, and Traits](structs-enums-and-traits.md)
- [Closures and Iterators](closures-and-iterators.md)
