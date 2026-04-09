# Functions and Control Flow

> Defining functions, expression-based control flow, pattern matching, and loops.

## Functions

All function signatures must be fully type-annotated. No return type inference.

```sploosh
fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn greet(name: &str) -> String {
    format("Hello, {}!", name)
}
```

## If / Else (Expression-Based)

`if`/`else` is an expression that returns a value. All branches must return the same type.

```sploosh
let status = if score > 90 {
    "excellent"
} else if score > 70 {
    "good"
} else {
    "needs work"
};
```

## Match (Exhaustive Pattern Matching)

```sploosh
match result {
    Ok(user) => process(user),
    Err(AppError::NotFound) => log("missing"),
    Err(AppError::Timeout { after }) => retry(after),
    Err(e) => return Err(e),
}

// Guards
match age {
    n if n < 13 => "child",
    n if n < 20 => "teen",
    n if n < 65 => "adult",
    _ => "senior",
}
```

Match must be exhaustive. Use `_` as a catch-all. All arms must return the same type.

## If Let and While Let

```sploosh
if let Some(user) = find_user(42) {
    process(user);
} else {
    log("user not found");
}

while let Ok(msg) = connection.read() {
    handle(msg);
}
```

## Loops

```sploosh
for item in collection { process(item); }       // iterate
for i in 0..10 { log(i); }                      // range
for (i, val) in items.iter() |> enumerate() { }  // with index
while connection.is_alive() { }                  // conditional
loop { if done { break; } }                      // infinite with break
```

## Destructuring in Bindings

```sploosh
let (x, y) = get_point();
let User { name, age, .. } = user;    // .. ignores remaining fields
let (Point { x, y }, radius) = get_circle();  // nested
```

## Next Steps

- [Ownership and Borrowing](ownership-and-borrowing.md)
- [Error Handling](error-handling.md)
