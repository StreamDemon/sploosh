# Basic Types and Variables

> Primitives, bindings, type annotations, and inference rules.

## Variable Bindings

```sploosh
let name = "Alice";              // immutable by default
let mut counter = 0;             // mutable binding
const MAX_RETRIES: u32 = 3;     // compile-time constant
```

## Primitive Types

```
i8  i16  i32  i64  i128    // Signed integers
u8  u16  u32  u64  u128    // Unsigned integers
f32  f64                    // Floating point
bool                        // true | false
char                        // Unicode scalar value
str                         // String slice (immutable reference)
String                      // Owned, growable string
()                          // Unit type (void equivalent)
```

## Default Numeric Types

- Unsuffixed integer literals default to `i64`
- Unsuffixed float literals default to `f64`
- Use suffixes to override: `42u32`, `3.14f32`

```sploosh
let x = 42;            // i64 (default)
let y = 3.14;          // f64 (default)
let z: u8 = 255;       // explicit annotation
let w = 100u32;         // suffix override
```

## Compound Types

```sploosh
let arr: [i32; 3] = [1, 2, 3];           // Fixed-size array
let list: Vec<i64> = vec![1, 2, 3];      // Growable list
let pair: (String, i64) = ("age".into(), 30);  // Tuple
let map: Map<String, i64> = Map::new();  // Hash map
let set: Set<i64> = Set::new();          // Hash set
```

## Option and Result

```sploosh
let maybe: Option<i64> = Some(42);       // Optional value
let outcome: Result<i64, AppError> = Ok(42);  // Fallible result
```

There is no `null`, `nil`, or `undefined` in Sploosh. Use `Option<T>` for optional values.

## Type Inference

Sploosh uses local type inference within function bodies. Function signatures must always be fully annotated.

```sploosh
let items = vec![1, 2, 3];              // Vec<i64> inferred
let names: Vec<String> = Vec::new();    // annotation needed (empty collection)
```

## Constants

```sploosh
const MAX_RETRIES: u32 = 3;
const TIMEOUT_MS: u64 = 30 * 1000;     // arithmetic on literals allowed
const API_URL: &str = "https://api.example.com";
const EMPTY: Vec<i32> = Vec::new();     // known constructors allowed
```

Constants are inlined at every usage site. No `static` keyword exists -- all mutable state lives in actors.

## Next Steps

- [Functions and Control Flow](functions-and-control-flow.md)
- [Ownership and Borrowing](ownership-and-borrowing.md)
