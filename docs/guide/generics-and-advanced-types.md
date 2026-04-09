# Generics and Advanced Types

> Where clauses, generic actors, trait objects, object safety, and type aliases.

## Basic Generics

```sploosh
fn first<T>(items: &[T]) -> Option<&T> {
    if items.len() == 0 { return None; }
    Some(&items[0])
}
```

## Trait Bounds

```sploosh
fn send_data<T: Serialize + Clone>(item: T) -> Result<(), NetError> {
    let bytes = item.to_bytes();
    network::transmit(bytes)
}
```

## Where Clauses

For complex bounds:

```sploosh
fn merge<A, B>(a: A, b: B) -> Vec<u8>
where
    A: Serialize,
    B: Serialize + Hash,
{ /* ... */ }
```

## Bounds on Struct Generics

```sploosh
struct Logger<T: Printable + Send> {
    items: Vec<T>,
}
```

## Dynamic Dispatch (Trait Objects)

```sploosh
fn print_area(shape: &dyn Shape) { /* runtime dispatch */ }
fn make_shape(kind: &str) -> Box<dyn Shape> { /* heap-allocated */ }
let shapes: Vec<Box<dyn Shape>> = vec![/* heterogeneous */];
```

**Object safety requirements:**
- No methods returning `Self`
- No methods with generic type parameters
- All methods take `&self`, `&mut self`, or `self`

**When to use which:**
- `T: Trait` (generics) -- zero-cost, monomorphized at compile time. Preferred.
- `dyn Trait` -- runtime dispatch. Use for heterogeneous collections or when concrete type is unknowable.

## Type Aliases

```sploosh
type UserId = u64;
type Outcome<T> = Result<T, AppError>;
```

## Turbofish

Resolves ambiguity in generic function calls:

```sploosh
let names = items.iter() |> collect::<Vec<String>>();
```

## Nested Generics

Fully supported: `Handle<Cache<String, Vec<Option<User>>>>`

## Next Steps

- [Strings and Formatting](strings-and-formatting.md)
- [Actors and Concurrency](actors-and-concurrency.md)
