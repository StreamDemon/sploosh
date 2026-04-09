# Type Conversion Rules

> No implicit conversions. All type conversions are explicit.

## From / Into Traits

```sploosh
let s: String = "hello".into();             // &str -> String via Into
let s: String = String::from("hello");      // explicit constructor
let slice: &str = &s;                       // String -> &str via auto-deref
```

## Numeric Casts

Use the `as` keyword for explicit numeric type conversions between integer types,
float types, and integer-to-float or float-to-integer.

### Rules

- **Widening** (e.g., `i32` -> `i64`): lossless, value is preserved.
- **Narrowing** (e.g., `i64` -> `i32`): truncates to the lower bits of the source value.
- **Float -> Int** (e.g., `f64` -> `i32`): truncates toward zero. Values exceeding the target range saturate to the min/max of the target type.
- **Int -> Float** (e.g., `i64` -> `f64`): nearest representable float; large values may lose precision.
- **Float -> Float** (e.g., `f64` -> `f32`): rounds to nearest representable value.

### Examples

```sploosh
let x: i64 = 42_i32 as i64;       // widening: 42
let y: i32 = 3_000_000_000_i64 as i32;  // narrowing: truncates to lower 32 bits (-1294967296)
let z: i32 = 3.9_f64 as i32;      // float-to-int: truncates toward zero -> 3
let w: f64 = 7_i32 as f64;        // int-to-float: 7.0
let v: f32 = 3.14_f64 as f32;     // float narrowing: rounds to nearest f32
```

### NOT for Non-Numeric Conversions

`as` only works between numeric types. For all other conversions, use `From` / `Into`:

```sploosh
// WRONG: let s: String = 42 as String;
// RIGHT:
let s: String = String::from(42);
let s: String = 42.into();
```

## @error Auto-Generated Conversions

Tuple variants in `@error` enums auto-generate `From` impls:

```sploosh
@error
enum AppError {
    Database(DbError),      // auto: impl From<DbError> for AppError
    Network(NetError),      // auto: impl From<NetError> for AppError
}
```

Only one `From` impl per source type is allowed.

## What Does NOT Convert Implicitly

- No integer widening (`i32` does not auto-promote to `i64`)
- No float-to-int or int-to-float
- No bool-to-int
- No null coercion (there is no null)
