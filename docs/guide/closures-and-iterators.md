# Closures and Iterators

> Closure capture semantics, closure traits, the Iter protocol, and lazy evaluation.

## Closures

Closures capture variables from their enclosing scope. Capture mode is determined by usage:

| Capture | When | Trait |
|---------|------|-------|
| `&T` (immutable borrow) | Closure only reads the variable | `Fn` |
| `&mut T` (mutable borrow) | Closure modifies the variable | `FnMut` |
| `move` (take ownership) | Explicit `move` keyword | `FnOnce` |

```sploosh
let name = String::from("Alice");
let greet = |prefix: &str| format("{} {}", prefix, name);  // borrows name

let mut counter = 0;
let mut inc = || { counter = counter + 1; };                // mut borrows counter

let data = vec![1, 2, 3];
let f = move || process(data);                              // moves data
```

**Rules:**
- `move` is required when a closure is passed to `spawn`, `send`, or returned from a function.
- `Fn` can be called multiple times. `FnMut` can be called multiple times. `FnOnce` can only be called once.

## Closure Type Annotations

```sploosh
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
    f(x)
}
```

## The Iter Trait

```sploosh
trait Iter {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

All iterators are lazy -- adaptors produce new iterators without consuming elements until a terminal runs.

## Adaptors (Lazy)

`map`, `filter`, `flat_map`, `take`, `skip`, `zip`, `chain`, `enumerate`, `peekable`

## Terminals (Eager)

`collect`, `fold`, `for_each`, `count`, `any`, `all`, `find`, `first`, `last`, `min`, `max`, `sum`

## Using Iterators

```sploosh
let names: Vec<String> = users.iter()
    .filter(|u| u.active)
    .map(|u| u.name.clone())
    .collect();

// Or with pipes (equivalent)
let names: Vec<String> = users.iter()
    |> filter(|u| u.active)
    |> map(|u| u.name.clone())
    |> collect();
```

## Consuming vs Borrowing

- `.iter()` borrows -- original collection still valid
- `for x in val` moves -- consumes the collection
- `.iter_mut()` borrows mutably

## Next Steps

- [Pipe Operator](pipe-operator.md)
- [Actors and Concurrency](actors-and-concurrency.md)
