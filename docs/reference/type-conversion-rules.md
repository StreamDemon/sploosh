# Type Conversion Rules

> No implicit conversions. All type conversions are explicit.

## From / Into Traits

```sploosh
let s: String = "hello".into();             // &str -> String via Into
let s: String = String::from("hello");      // explicit constructor
let slice: &str = &s;                       // String -> &str via auto-deref
```

## Numeric Casts

<!-- TODO: Document numeric casting rules (as keyword or conversion functions) once finalized -->

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
