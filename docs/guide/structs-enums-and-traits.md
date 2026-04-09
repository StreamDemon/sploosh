# Structs, Enums, and Traits

> Custom types, algebraic data types, trait system, generics, and dynamic dispatch.

## Structs

```sploosh
struct User {
    name: String,
    age: u32,
    role: Role,
}

// Type alias
type UserId = u64;
```

## Enums (Algebraic Data Types)

```sploosh
enum Role {
    Admin,
    Editor { level: u8 },
    Viewer,
}

enum Shape {
    Circle { radius: f64 },
    Rect { width: f64, height: f64 },
    Point,
}
```

## Traits

```sploosh
trait Serialize {
    fn to_bytes(&self) -> Vec<u8>;
    fn size_hint(&self) -> u64 { 0 }   // default implementation
}

impl Serialize for User {
    fn to_bytes(&self) -> Vec<u8> { /* ... */ }
}
```

## Supertraits

```sploosh
trait Loggable: Printable {
    fn log_level(&self) -> &str;
}
```

Implementors of `Loggable` must also implement `Printable`. Multiple supertraits use `+`: `trait Storable: Serialize + Clone + Send { }`

## Generics and Trait Bounds

```sploosh
fn send_data<T: Serialize + Clone>(item: T) -> Result<(), NetError> {
    let bytes = item.to_bytes();
    network::transmit(bytes)
}

// Where clause for complex bounds
fn merge<A, B>(a: A, b: B) -> Vec<u8>
where
    A: Serialize,
    B: Serialize + Hash,
{ /* ... */ }
```

## Dynamic Dispatch (Trait Objects)

When the concrete type isn't known at compile time:

```sploosh
fn print_area(shape: &dyn Shape) { /* ... */ }
fn make_shape(kind: &str) -> Box<dyn Shape> { /* ... */ }
let shapes: Vec<Box<dyn Shape>> = vec![/* ... */];
```

**Object safety rules:** No methods returning `Self`, no generic type parameters on methods, all methods must take `&self`/`&mut self`/`self`.

Prefer generics (`T: Trait`) for zero-cost monomorphization. Use `dyn Trait` for heterogeneous collections or plugin architectures.

## Derives

```sploosh
@derive(Debug, Clone, Eq, Hash)
struct Point { x: f64, y: f64 }
```

Available: `Debug`, `Clone`, `Copy`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Ord`.

## Next Steps

- [Error Handling](error-handling.md)
- [Generics and Advanced Types](generics-and-advanced-types.md)
