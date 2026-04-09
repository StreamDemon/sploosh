# SPLOOSH Language Specification v0.3.0-draft

> **AI-Native · Systems-Grade · Web2/Web3 Dual-Target**
>
> A programming language designed for LLM generation accuracy, combining Rust-level safety
> and performance with Elixir-style concurrency — built entirely from syntax primitives
> deeply trained across all major language models.

---

## 1. Design Principles

1. **One way to do everything.** Every operation has exactly one syntactic form. Zero ambiguity.
2. **Familiar vocabulary only.** Every keyword and operator is drawn from the top 12 most-trained languages. No novel tokens.
3. **Explicit over implicit.** No implicit conversions, no hidden control flow, no operator overloading.
4. **Errors are values.** All fallible operations return `Result<T, E>`. No exceptions. No panics in safe code.
5. **Concurrency is structural.** Actor-based isolation with message passing. No shared mutable state.
6. **Dual-target by design.** Single source compiles to native (LLVM), WASM (web2), and on-chain bytecode (web3).
7. **Spec fits in a prompt.** The full language core is describable in under 4,000 tokens for LLM system prompts.

---

## 2. Lexical Structure

### 2.1 Character Set

Source files are UTF-8. All keywords, operators, and identifiers use ASCII only.

### 2.2 Comments

```
// Line comment
/// Doc comment (attaches to next item)
```

Block comments are intentionally omitted. One way to comment.

### 2.3 Keywords (38 total)

**Declarations:**
`fn` `let` `const` `type` `struct` `enum` `trait` `impl` `mod` `use` `pub`

**Control Flow:**
`if` `else` `match` `for` `in` `while` `loop` `break` `continue` `return`

**Types & Values:**
`self` `Self` `true` `false` `none`

**Concurrency:**
`actor` `send` `recv` `spawn` `async` `await` `select`

**Closures:**
`move`

**Web3:**
`onchain` `offchain` `storage` `emit`

### 2.4 Operators (precedence high → low)

| Prec | Operator     | Meaning                    | Assoc |
|------|-------------|----------------------------|-------|
| 14   | `.`          | Field/method access        | Left  |
| 13   | `()`  `[]`   | Call, Index                | Left  |
| 12   | `?`          | Error propagation          | Post  |
| 11   | `!`          | Logical NOT                | Pre   |
| 10   | `*` `/` `%`  | Multiply, Divide, Modulo   | Left  |
| 9    | `+` `-`      | Add, Subtract              | Left  |
| 8    | `|>`         | Pipe                       | Left  |
| 7    | `<` `>` `<=` `>=` | Comparison            | Left  |
| 6    | `==` `!=`    | Equality                   | Left  |
| 5    | `&&`         | Logical AND                | Left  |
| 4    | `\|\|`       | Logical OR                 | Left  |
| 3    | `..` `..=`   | Range, Inclusive Range     | None  |
| 2    | `=`          | Assignment                 | Right |
| 1    | `=>`         | Match arm / Lambda         | Right |
| 0    | `->`         | Return type annotation     | Right |

### 2.5 Sigils

| Sigil | Meaning                       |
|-------|-------------------------------|
| `&`   | Immutable reference (borrow)  |
| `&mut`| Mutable reference (borrow)    |
| `@`   | Attribute / decorator         |
| `#`   | Compiler directive            |
| `::`  | Path separator / type access  |
| `:`   | Type annotation               |

---

## 3. Type System

### 3.1 Primitive Types

```
i8  i16  i32  i64  i128    // Signed integers
u8  u16  u32  u64  u128    // Unsigned integers
f32  f64                    // Floating point
bool                        // true | false
char                        // Unicode scalar value
str                         // String slice (immutable)
String                      // Owned, growable string
()                          // Unit type (void equivalent)
```

### 3.2 Compound Types

```
[T; N]              // Fixed-size array
[T]                 // Slice
Vec<T>              // Growable list
Map<K, V>           // Hash map
Set<T>              // Hash set
(T, U, V)           // Tuple
Option<T>           // Some(T) | None
Result<T, E>        // Ok(T) | Err(E)
```

### 3.3 Custom Types

```sploosh
// Struct
struct User {
    name: String,
    age: u32,
    role: Role,
}

// Enum (algebraic data type)
enum Role {
    Admin,
    Editor { level: u8 },
    Viewer,
}

// Type alias
type UserId = u64;
type Outcome<T> = Result<T, AppError>;
```

### 3.4 Generics

```sploosh
fn first<T>(items: &[T]) -> Option<&T> {
    if items.len() == 0 {
        return None;
    }
    Some(&items[0])
}
```

### 3.5 Trait System

```sploosh
trait Serialize {
    fn to_bytes(&self) -> Vec<u8>;
    fn size_hint(&self) -> u64 { 0 }   // Default implementation
}

impl Serialize for User {
    fn to_bytes(&self) -> Vec<u8> {
        // ...
    }
}
```

**Supertraits:** A trait can require that implementors also implement other traits:

```sploosh
trait Printable {
    fn to_display(&self) -> String;
}

// Loggable requires Printable — implementors must impl both
trait Loggable: Printable {
    fn log_level(&self) -> &str;

    // Default impl can use supertrait methods
    fn log(&self) {
        print(format("[{}] {}", self.log_level(), self.to_display()));
    }
}

impl Printable for User {
    fn to_display(&self) -> String { format("User({})", self.name) }
}

impl Loggable for User {
    fn log_level(&self) -> &str { "INFO" }
    // log() uses default impl from trait
}
```

Multiple supertraits use `+`: `trait Storable: Serialize + Clone + Send { }`

### 3.6 Trait Bounds

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
{
    // ...
}

// Trait bounds on struct generics
struct Logger<T: Printable + Send> {
    items: Vec<T>,
}
```

### 3.7 Type Unification Rules

All branches of a conditional expression must return the same type:

```sploosh
// All match arms must unify to the same type
match value {
    Ok(n) => n * 2,         // i64
    Err(_) => 0,            // i64 — OK, same type
}

// if/else arms must unify
let x = if flag { 42 } else { 0 };   // Both i64 — OK
```

Pattern bindings in match arms follow these rules:
- Primitives (`i32`, `f64`, `bool`, etc.) are **copied** into the binding.
- Non-Copy types (`String`, `Vec<T>`, etc.) are **moved** into the binding.
- Use `ref` to borrow instead of move: `Some(ref name) => name.len()`.
- Struct/enum field destructuring binds by move unless `ref` is used.

```sploosh
match user.role {
    Role::Admin => "admin",
    Role::Editor { ref level } => format("editor-{}", level),  // borrows level
    Role::Viewer => "viewer",
}
```

When matching on `&self` or `&T`, pattern bindings are automatically references.
No explicit `ref` needed.

### 3.8 Type Inference

Sploosh uses local type inference within function bodies. Function signatures
(parameters, return types) must always be fully annotated.

**Default numeric types:**
- Unsuffixed integer literals default to `i64`.
- Unsuffixed float literals default to `f64`.
- Use suffixes to override: `42u32`, `3.14f32`.

```sploosh
let x = 42;            // i64 (default)
let y = 3.14;          // f64 (default)
let z: u8 = 255;       // explicit annotation
let w = 100u32;         // suffix

let items = vec![1, 2, 3];              // Vec<i64>
let names: Vec<String> = Vec::new();    // annotation needed (empty collection)
```

**Inference rules:**
- Type flows forward from initializer: `let x = expr` infers x's type from expr.
- Type flows backward from usage: `let items = Vec::new(); items.push(42u8);` infers `Vec<u8>`.
- Turbofish (`::<T>`) resolves ambiguity: `collect::<Vec<String>>()`.
- No inference across function boundaries — all `fn` signatures are explicit.
- Nested generics are fully supported: `Handle<Cache<String, Vec<Option<User>>>>`.

### 3.9 Dynamic Dispatch (Trait Objects)

When the concrete type isn't known at compile time, use trait objects with `dyn`:

```sploosh
trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

// Borrowed trait object
fn print_area(shape: &dyn Shape) {
    print(format("{}: {:.2}", shape.name(), shape.area()));
}

// Owned trait object (heap-allocated)
fn make_shape(kind: &str) -> Box<dyn Shape> {
    match kind {
        "circle" => Box::new(Circle { radius: 5.0 }),
        "rect" => Box::new(Rect { width: 3.0, height: 4.0 }),
        _ => Box::new(Point {}),
    }
}

// Heterogeneous collections
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rect { width: 3.0, height: 4.0 }),
];

for shape in shapes.iter() {
    print_area(shape.as_ref());
}
```

**Object safety:** A trait can be used as `dyn Trait` only if:
- No methods return `Self` (the concrete type is erased).
- No methods have generic type parameters.
- All methods take `&self`, `&mut self`, or `self` as first parameter.

**When to use which:**
- `T: Trait` (generics) — zero-cost, monomorphized at compile time. Preferred.
- `dyn Trait` — runtime dispatch, one copy of code. Use for heterogeneous collections,
  plugin architectures, or when the concrete type is unknowable at compile time.

---

## 4. Ownership & Borrowing

Sploosh uses a simplified Rust-like ownership model. **All lifetimes are explicit when needed.**
The compiler enforces single-owner semantics with borrowing.

### 4.1 Rules

1. Every value has exactly one owner.
2. When the owner goes out of scope, the value is dropped.
3. You may have either ONE `&mut` reference OR any number of `&` references. Never both.
4. References must always be valid (no dangling pointers).

### 4.2 Move vs Copy

```sploosh
let a = String::from("hello");
let b = a;          // a is MOVED to b. a is no longer valid.

let x: i32 = 42;
let y = x;          // x is COPIED. Both valid. (primitives implement Copy)
```

### 4.3 Borrowing

```sploosh
fn greet(name: &str) -> String {
    format("Hello, {}", name)
}

fn update_name(user: &mut User, new_name: String) {
    user.name = new_name;
}

let user = User { name: "Alice".into(), age: 30, role: Role::Viewer };
greet(&user.name);
```

### 4.4 Explicit Lifetimes

```sploosh
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

### 4.5 Closures and Capture Semantics

Closures capture variables from their enclosing scope. Capture mode is determined by usage:

1. **Immutable borrow (`&T`)** — default when the closure only reads the variable.
2. **Mutable borrow (`&mut T`)** — when the closure modifies the variable.
3. **Move** — when the closure takes ownership. Use the `move` keyword explicitly.

```sploosh
let name = String::from("Alice");

// Immutable borrow capture (reads name)
let greet = |prefix: &str| format("{} {}", prefix, name);
greet("Hello");
print(name);          // Still valid — only borrowed

// Mutable borrow capture (modifies counter)
let mut counter = 0;
let mut inc = || { counter = counter + 1; };
inc();
inc();
assert(counter == 2);

// Move capture (closure takes ownership)
let data = vec![1, 2, 3];
let handle = spawn move || {
    // data is moved into the spawned task
    process(data);
};
// data is no longer valid here
```

**Rules:**
- A closure that captures `&mut` cannot be called while any other reference to the captured variable exists.
- A `move` closure takes ownership of all captured variables. The originals become invalid.
- If a closure is passed to `spawn`, `send`, or returned from a function, it **must** be `move`.
- Closures implement `Fn`, `FnMut`, or `FnOnce` traits based on capture:
  - `Fn` — captures only `&T`. Can be called multiple times.
  - `FnMut` — captures `&mut T`. Can be called multiple times.
  - `FnOnce` — captures by move. Can only be called once.

```sploosh
// Closure type annotations when needed
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
    f(x)
}

let double = |n: i64| n * 2;
let result = apply(double, 21);  // 42
```

### 4.6 Constants

```sploosh
// Module-level constants — value must be computable at compile time
const MAX_RETRIES: u32 = 3;
const TIMEOUT_MS: u64 = 30 * 1000;     // arithmetic on literals is allowed
const API_URL: &str = "https://api.example.com";
const EMPTY: Vec<i32> = Vec::new();     // constructors of known types allowed

// NOT allowed in const:
// const BAD: String = format("hi");    // function calls (except constructors)
// const BAD: Config = load_config();   // runtime function calls
```

**Const rules:**
- `const` values are inlined at every usage site (no address, no allocation).
- Expressions: literals, arithmetic on literals, known constructors (`Vec::new()`, `Map::new()`), struct/enum construction from other consts.
- No `const fn` — keep it simple. If you need computed values, use `let` in a function.
- No `static` keyword — there is no module-level mutable state.
  All mutable state lives in actors. One way to do it.

---

## 5. Control Flow

### 5.1 If / Else (expression-based)

```sploosh
let status = if score > 90 {
    "excellent"
} else if score > 70 {
    "good"
} else {
    "needs work"
};
```

### 5.2 Match (exhaustive pattern matching)

```sploosh
match result {
    Ok(user) => process(user),
    Err(AppError::NotFound) => log("missing"),
    Err(AppError::Timeout { after }) => retry(after),
    Err(e) => return Err(e),
}

// Destructuring
match point {
    (0, 0) => "origin",
    (x, 0) => format("{} on x-axis", x),
    (0, y) => format("{} on y-axis", y),
    (x, y) => format("({}, {})", x, y),
}

// Guards
match age {
    n if n < 13 => "child",
    n if n < 20 => "teen",
    n if n < 65 => "adult",
    _ => "senior",
}
```

**Match rules:**
- All arms must return the same type (see §3.7).
- Match must be exhaustive. Use `_` as a catch-all.
- Pattern bindings follow move/copy/ref rules (see §3.7).
- Matching on `&self` in trait impls is allowed and idiomatic:

```sploosh
impl Display for Shape {
    fn to_string(&self) -> String {
        match self {
            Shape::Circle { radius } => format("circle r={}", radius),
            Shape::Rect { width, height } => format("rect {}x{}", width, height),
            Shape::Point => "point".into(),
        }
    }
}
```

When matching on `&self` or `&T`, pattern bindings are automatically references.
No explicit `ref` needed.

### 5.3 Destructuring in Let Bindings

Patterns can be used in `let` bindings for convenient unpacking:

```sploosh
// Tuple destructuring
let (x, y) = get_coordinates();
let (name, age, _) = get_user_tuple();  // _ discards

// Struct destructuring
let User { name, age, .. } = get_user();    // .. ignores remaining fields
let Point { x, y } = origin;

// Enum destructuring (irrefutable only — must match all variants or use if-let)
// This is a compile error because Option has two variants:
// let Some(value) = maybe_value;  // ERROR: refutable pattern in let

// Nested destructuring
let (Point { x, y }, radius) = get_circle();
```

**Irrefutable vs refutable patterns:**
- `let` bindings require **irrefutable** patterns — patterns that always match.
  Tuples, structs, and single-variant enums are irrefutable.
- `match` arms and `if let` / `while let` accept **refutable** patterns —
  patterns that might not match.

### 5.4 If Let and While Let

For cases where you want to match a single pattern without a full `match`:

```sploosh
// if let — executes block only if pattern matches
if let Some(user) = find_user(42) {
    process(user);
} else {
    log("user not found");
}

// if let with enum variants
if let Ok(config) = load_config("app.toml") {
    start_server(config);
}

// Nested if let
if let Some(user) = find_user(42) {
    if let Role::Admin = user.role {
        grant_access();
    }
}

// while let — loops while pattern matches
while let Some(item) = queue.pop() {
    process(item);
}

// while let with Result
while let Ok(msg) = connection.read() {
    handle(msg);
}
```

### 5.5 Loops

```sploosh
// Iterate (primary loop form)
for item in collection {
    process(item);
}

// Destructuring in for loops
for (index, value) in items.iter() |> enumerate() {
    print(format("{}: {}", index, value));
}

for User { name, age, .. } in users {
    print(format("{} is {}", name, age));
}

// Range iteration
for i in 0..10 {
    log(i);
}

// While
while connection.is_alive() {
    let msg = connection.read()?;
    handle(msg);
}

// Infinite loop with break
loop {
    let event = poll();
    if event.is_shutdown() {
        break;
    }
}
```

### 5.6 Pipe Operator

The pipe operator passes the left-hand value as the **first argument** to the
right-hand function or method:

```sploosh
// Single-argument functions
let result = raw_input |> parse_json |> validate |> serialize;
// Equivalent to: serialize(validate(parse_json(raw_input)))

// Multi-argument functions: piped value becomes first argument
fn add(a: i64, b: i64) -> i64 { a + b }
let result = 10 |> add(5);     // add(10, 5) = 15
let result = 10
    |> add(5)                    // add(10, 5) = 15
    |> add(20);                  // add(15, 20) = 35
```

**Pipe rules:**
- `x |> f` desugars to `f(x)`.
- `x |> f(a, b)` desugars to `f(x, a, b)`. Piped value is always the **first** argument.
- `x |> obj.method(a)` desugars to `obj.method(x, a)` — but this is unusual.
  For method chains, prefer: `x.method(a)`.
- There is no placeholder syntax (`_`). If you need the piped value in a
  position other than first, use a closure:

```sploosh
// Piped value as second argument — use a closure
let result = 10 |> (|v| multiply(3, v));   // multiply(3, 10) = 30
```

- For iterator methods on `.iter()`, pipe and method chains are equivalent:

```sploosh
// These are identical:
let names = users.iter().filter(|u| u.active).map(|u| u.name.clone()).collect();
let names = users.iter() |> filter(|u| u.active) |> map(|u| u.name.clone()) |> collect();
```

When used with `.iter()`, `|> method(args)` desugars to `.method(args)`.

### 5.7 Pipe + Error Propagation Rules

The `?` operator (precedence 12) binds tighter than `|>` (precedence 8). When pipe
chains involve `Result<T, E>` returns, use `?` on each fallible stage to unwrap
before piping to the next:

```sploosh
// CORRECT: ? unwraps each Result, then pipes the Ok value forward
let report = raw_input
    |> parse_json?        // parse_json(raw_input) -> Result → unwrap or return Err
    |> validate?          // validate(parsed) -> Result → unwrap or return Err
    |> transform?;        // transform(valid) -> Result → unwrap or return Err

// Evaluation order:
//   1. parse_json(raw_input) → Result<Json, E>
//   2. ? unwraps → Json (or early-returns Err)
//   3. validate(json) → Result<Valid, E>
//   4. ? unwraps → Valid (or early-returns Err)
//   5. transform(valid) → Result<Report, E>
//   6. ? unwraps → Report (or early-returns Err)
```

**Rules:**
- `expr |> f?` is parsed as `(expr |> f)?`, which means `f(expr)?`.
- When all functions return `Result`, use `?` on every stage.
- When functions return plain values (non-Result), omit `?`.
- Mixed chains:

```sploosh
let output = raw_input
    |> trim                 // &str -> &str (infallible, no ?)
    |> parse_json?          // &str -> Result<Json, E> (fallible, needs ?)
    |> extract_name;        // Json -> String (infallible, no ?)
```

- For `Option<T>` chains, the same pattern works — `?` returns `None` early:

```sploosh
let email = find_user(42)?
    |> get_profile?
    |> get_email;
```

---

## 6. Error Handling

### 6.1 Result Type (mandatory handling)

```sploosh
fn read_config(path: &str) -> Result<Config, FileError> {
    let content = fs::read(path)?;          // ? propagates Err early
    let parsed = json::parse(&content)?;    // ? again
    Ok(Config::from(parsed))
}
```

### 6.2 Custom Error Types

```sploosh
enum AppError {
    NotFound { resource: String },
    Unauthorized,
    Database(DbError),
    Network(NetError),
    Validation { field: String, message: String },
}

// Automatic conversion via From trait
impl From<DbError> for AppError {
    fn from(e: DbError) -> Self {
        AppError::Database(e)
    }
}
```

### 6.3 The `@error` Derive Macro

For common error enum patterns, `@error` auto-generates `From` impls and display strings:

```sploosh
@error
enum AppError {
    NotFound { resource: String },          // Display: "not found: {resource}"
    Unauthorized,                           // Display: "unauthorized"
    Database(DbError),                      // From<DbError> auto-generated
    Network(NetError),                      // From<NetError> auto-generated
    Validation { field: String, msg: String }, // Display: "validation: {field}: {msg}"
}

// The @error attribute generates:
// - impl From<DbError> for AppError
// - impl From<NetError> for AppError
// - impl Display for AppError
// - impl Error for AppError
```

**Rules for `@error`:**
- Tuple variants like `Database(DbError)` generate `From<DbError>` impls.
- Struct variants generate a Display string from their field names.
- Unit variants use their snake_case name as the display string.
- Only one `From` impl per source type is allowed (no ambiguity).

### 6.4 Error Context and Chaining

Errors can carry context via the `context` method (available on all `Result` types):

```sploosh
fn load_config(path: &str) -> Result<Config, AppError> {
    let content = fs::read(path)
        .context(format("failed to read config from {}", path))?;
    
    let parsed = json::parse(&content)
        .context("invalid JSON in config file")?;
    
    Ok(Config::from(parsed))
}
```

The `context` method wraps the original error, preserving the chain for debugging.
When printed, errors display as: `"failed to read config from ./app.json: file not found"`.

### 6.5 Option Type

```sploosh
fn find_user(id: UserId) -> Option<User> {
    let row = db.query("users", id);
    match row {
        Some(data) => Some(User::from(data)),
        None => None,
    }
}

// Chaining with map/and_then
let email = find_user(42)
    |> map(|u| u.email)
    |> unwrap_or("unknown@example.com".into());
```

### 6.6 No Null. No Exceptions.

There is no `null`, `nil`, or `undefined` in Sploosh. Use `Option<T>` for optional values.
There is no `throw` or `try/catch`. Use `Result<T, E>` for fallible operations.
This is enforced at the compiler level.

---

## 7. Iterators and Collections

### 7.1 The Iter Trait

Any type that implements the `Iter` trait can be used in `for` loops and pipe chains:

```sploosh
trait Iter {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

`Vec<T>`, `Map<K, V>`, `Set<T>`, `[T; N]`, and ranges (`0..10`) all implement `Iter`.

### 7.2 Iterator Adaptors

Iterator adaptors are **methods on the Iter trait**. They are lazy —
they produce a new iterator without consuming elements until a terminal operation runs.

**Adaptors (lazy):**

| Method | Purpose |
|---|---|
| `map(f)` | Transform each element |
| `filter(f)` | Keep elements where predicate is true |
| `flat_map(f)` | Map then flatten |
| `take(n)` | First n elements |
| `skip(n)` | Skip first n elements |
| `zip(other)` | Pair elements from two iterators |
| `chain(other)` | Concatenate two iterators |
| `enumerate()` | Pair each element with its index |
| `peekable()` | Allow peeking at next without consuming |

**Terminals (eager, consume the iterator):**

| Method | Purpose |
|---|---|
| `collect::<C>()` | Gather into a collection |
| `fold(init, f)` | Reduce to a single value |
| `for_each(f)` | Run a function on each element |
| `count()` | Count elements |
| `any(f)` / `all(f)` | Boolean test across elements |
| `find(f)` | First element matching predicate |
| `first()` / `last()` | First or last element |
| `min()` / `max()` | Minimum or maximum (requires `Ord`) |
| `sum()` | Sum all elements (requires `Add`) |

### 7.3 Using Iterators with Pipes

The pipe operator and method chains are interchangeable for iterator operations:

```sploosh
// Method chain style
let names: Vec<String> = users.iter()
    .filter(|u| u.active)
    .map(|u| u.name.clone())
    .collect();

// Pipe style (equivalent)
let names: Vec<String> = users.iter()
    |> filter(|u| u.active)
    |> map(|u| u.name.clone())
    |> collect();
```

In pipe style, `|> method(args)` desugars to `.method(args)`.

**Consuming vs borrowing iteration:**

```sploosh
let items = vec![1, 2, 3];

// .iter() borrows — items still valid after
for x in items.iter() { print(x); }
print(items.len());  // OK

// for..in consumes by default (moves)
for x in items { print(x); }
// items is no longer valid

// .iter_mut() for mutable borrowing
for x in items.iter_mut() { *x = *x + 1; }
```

### 7.4 Collection Methods

**Vec<T>:**
`push`, `pop`, `insert`, `remove`, `len`, `is_empty`, `contains`, `sort`, `sort_by`,
`reverse`, `dedup`, `clear`, `iter`, `iter_mut`, `get`, `first`, `last`

**Map<K, V>:**
`insert`, `remove`, `get`, `contains_key`, `len`, `is_empty`, `keys`, `values`,
`iter`, `entry`, `clear`

**Set<T>:**
`insert`, `remove`, `contains`, `len`, `is_empty`, `union`, `intersection`,
`difference`, `iter`, `clear`

---

## 8. Concurrency (Actor Model)

### 8.1 Actor Definition

```sploosh
actor Counter {
    state: i64,

    // Initialize actor state
    fn init(start: i64) -> Self {
        Counter { state: start }
    }

    // Handle incoming messages (one at a time, no data races)
    pub fn increment(&mut self, amount: i64) {
        self.state = self.state + amount;
    }

    pub fn get(&self) -> i64 {
        self.state
    }
}
```

### 8.2 Actor Handle Types and Message Ownership

`spawn` returns a typed handle: `Handle<ActorName>`. Handles are the only way to interact
with actors after spawning.

```sploosh
let counter: Handle<Counter> = spawn Counter::init(0);
```

**Handle rules:**
- `Handle<T>` implements `Clone` and `Send`. Handles can be freely copied and passed.
- Handles can be stored in structs, other actors, and collections.
- `&self` methods on handles are **request/reply** — blocks caller until response.
- `&mut self` methods can be called via `send` (fire-and-forget) or direct call (blocks).
- Sending to a dead actor: request/reply returns `Err(ActorError::Dead)`. `send` silently drops.

**Message ownership rules:**
All parameters to `pub` actor methods must be **owned types**. References (`&T`, `&mut T`)
are not allowed in actor method signatures because messages are delivered asynchronously —
the caller's stack frame may no longer exist when the actor processes the message.

```sploosh
actor Logger {
    entries: Vec<String>,
    fn init() -> Self { Logger { entries: Vec::new() } }

    // CORRECT: owned String
    pub fn log(&mut self, msg: String) { self.entries.push(msg); }

    // COMPILE ERROR: &str is a reference — not allowed in actor methods
    // pub fn log_ref(&mut self, msg: &str) { ... }

    pub fn count(&self) -> u64 { self.entries.len() as u64 }
}

actor Worker {
    logger: Handle<Logger>,    // Store a handle to another actor

    fn init(logger: Handle<Logger>) -> Self {
        Worker { logger }
    }

    pub fn do_work(&mut self, task: String) {
        send self.logger.log(format("Working on: {}", task));
    }
}

fn main() -> Result<(), AppError> {
    let logger = spawn Logger::init();
    let worker = spawn Worker::init(logger.clone());

    send worker.do_work("task_1".into());
    let count = logger.count();  // Request/reply — blocks
    Ok(())
}
```

**Private (non-pub) methods** within an actor can use references freely since they
are only called internally during message handling, when the actor's own scope is alive.

### 8.3 Generic Actors

Actors can be generic. All message types must implement `Send`.

```sploosh
actor Cache<K: Hash + Eq + Send, V: Clone + Send> {
    data: Map<K, V>,
    max_size: u64,

    fn init(max_size: u64) -> Self {
        Cache { data: Map::new(), max_size }
    }

    pub fn set(&mut self, key: K, value: V) {
        if self.data.len() as u64 >= self.max_size {
            if let Some(first_key) = self.data.keys().first().cloned() {
                self.data.remove(&first_key);
            }
        }
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).map(|v| v.clone())
    }
}
```

**EBNF:** `actor_def = [ attrs ] "actor" IDENT [ generics ] "{" { actor_item } "}" ;`

### 8.4 Spawning and Messaging

```sploosh
fn main() -> Result<(), AppError> {
    let counter = spawn Counter::init(0);

    send counter.increment(5);      // Fire-and-forget
    send counter.increment(3);

    let value = counter.get();       // Request/reply (blocks)
    assert(value == 8);

    Ok(())
}
```

### 8.5 Select (multiplexed receive)

```sploosh
select {
    msg = recv channel_a => handle_a(msg),
    msg = recv channel_b => handle_b(msg),
    _ = timeout(5000) => return Err(AppError::Timeout),
}
```

### 8.6 Supervision

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 3)
actor WorkerPool {
    children: Vec<Handle<Worker>>,

    fn init(size: u32) -> Self {
        let children = (0..size)
            |> map(|_| spawn Worker::init())
            |> collect;
        WorkerPool { children }
    }
}
```

### 8.7 Actor Failure and Recovery

Actors can fail due to runtime errors (out-of-bounds access, integer overflow in debug mode,
explicit `assert` failures). When an actor fails:

1. **The actor dies.** Its state is dropped. Its handle becomes dead.
2. **Pending messages are discarded.** Messages in the actor's mailbox are lost.
3. **Callers are notified.** Request/reply calls receive `Err(ActorError::Dead)`.
   Fire-and-forget `send` calls are silently dropped.
4. **Supervisors restart.** If the actor has a supervisor, the restart strategy applies.

```sploosh
@error
enum ActorError {
    Dead,                           // Actor has terminated
    Timeout,                        // Request/reply timed out
    PanicMessage { msg: String },   // What went wrong (for logging)
}

fn main() -> Result<(), AppError> {
    let worker = spawn Worker::init();

    // Worker might die during processing
    match worker.process(data) {
        Ok(result) => use_result(result),
        Err(ActorError::Dead) => {
            log::warn("Worker died, spawning replacement");
            let worker = spawn Worker::init();
            worker.process(data)?
        }
        Err(e) => return Err(AppError::from(e)),
    }

    Ok(())
}
```

**There is no `panic` keyword.** Actors die from runtime checks (bounds, overflow,
failed assertions), not from explicit panic calls. The "no panics in safe code" principle
means the language has no user-callable panic — runtime checks are the only source of
actor death.

### 8.8 Async/Await (for non-actor async)

```sploosh
async fn fetch_data(url: &str) -> Result<Response, NetError> {
    let conn = net::connect(url).await?;
    let response = conn.get("/api/data").await?;
    Ok(response)
}
```

---

## 9. String Formatting and Methods

### 9.1 The `format` Function

`format` is a **compiler intrinsic** that produces a `String` from a
template and arguments. It uses `{}` placeholders, resolved positionally:

```sploosh
let s = format("Hello, {}!", name);
let s = format("{} is {} years old", name, age);
let s = format("Pi is {:.4}", 3.14159);
let s = format("Hex: {:x}", 255);
let s = format("Debug: {:?}", some_value);
```

### 9.2 Format Specifiers

| Specifier | Meaning | Example |
|---|---|---|
| `{}` | Default display | `format("{}", 42)` → `"42"` |
| `{:?}` | Debug representation | `format("{:?}", vec)` → `"[1, 2, 3]"` |
| `{:.N}` | N decimal places (floats) | `format("{:.2}", 3.14159)` → `"3.14"` |
| `{:x}` / `{:X}` | Hex lower / upper | `format("{:x}", 255)` → `"ff"` |
| `{:b}` | Binary | `format("{:b}", 10)` → `"1010"` |
| `{:o}` | Octal | `format("{:o}", 8)` → `"10"` |
| `{:>N}` | Right-align, width N | `format("{:>10}", "hi")` → `"        hi"` |
| `{:<N}` | Left-align, width N | `format("{:<10}", "hi")` → `"hi        "` |
| `{:0N}` | Zero-pad, width N | `format("{:06}", 42)` → `"000042"` |

### 9.3 Display and Debug Traits

Types used with `{}` must implement `Display`. Types used with `{:?}` must implement `Debug`.
`@derive(Debug)` auto-generates `Debug`. `Display` must be implemented manually.

```sploosh
@derive(Debug)
struct Point { x: f64, y: f64 }

impl Display for Point {
    fn to_string(&self) -> String {
        format("({}, {})", self.x, self.y)
    }
}
```

### 9.4 No String Interpolation

There are no f-strings or template literals. `format()` is the only way to build
formatted strings. One way to do it.

### 9.5 String Methods

**`str` (immutable string slice) methods:**

| Method | Signature | Purpose |
|---|---|---|
| `len` | `fn len(&self) -> u64` | Byte length |
| `is_empty` | `fn is_empty(&self) -> bool` | True if length is 0 |
| `contains` | `fn contains(&self, pat: &str) -> bool` | Substring search |
| `starts_with` | `fn starts_with(&self, pat: &str) -> bool` | Prefix check |
| `ends_with` | `fn ends_with(&self, pat: &str) -> bool` | Suffix check |
| `find` | `fn find(&self, pat: &str) -> Option<u64>` | Index of first match |
| `trim` | `fn trim(&self) -> &str` | Strip leading/trailing whitespace |
| `trim_start` | `fn trim_start(&self) -> &str` | Strip leading whitespace |
| `trim_end` | `fn trim_end(&self) -> &str` | Strip trailing whitespace |
| `to_uppercase` | `fn to_uppercase(&self) -> String` | Returns new uppercased String |
| `to_lowercase` | `fn to_lowercase(&self) -> String` | Returns new lowercased String |
| `replace` | `fn replace(&self, from: &str, to: &str) -> String` | Replace all occurrences |
| `split` | `fn split(&self, pat: &str) -> Iter<&str>` | Split into iterator |
| `chars` | `fn chars(&self) -> Iter<char>` | Iterate over Unicode characters |
| `as_bytes` | `fn as_bytes(&self) -> &[u8]` | View as byte slice |

**`String` (owned, growable) additional methods:**

| Method | Signature | Purpose |
|---|---|---|
| `push_str` | `fn push_str(&mut self, s: &str)` | Append a string slice |
| `push` | `fn push(&mut self, c: char)` | Append a character |
| `clear` | `fn clear(&mut self)` | Empty the string |
| `into_bytes` | `fn into_bytes(self) -> Vec<u8>` | Convert to byte vector |

**String concatenation** — there is no `+` for strings (no operator overloading).
Use `format` or `push_str`:

```sploosh
// Preferred: format
let full = format("{} {}", first_name, last_name);

// Mutable building
let mut s = String::from("hello");
s.push_str(" world");
```

**Conversions:**

```sploosh
let s: String = "hello".into();             // &str → String via Into trait
let s: String = String::from("hello");      // explicit constructor
let slice: &str = &s;                       // String → &str via auto-deref
```

**Indexing:** String indexing is byte-based. `&s[0..5]` returns a `&str` of the first
5 bytes. For Unicode-safe character access, use `.chars()` iterator.

---

## 10. Module System

### 10.1 Module Declaration

```sploosh
// File: src/auth/mod.sp
mod auth {
    pub mod login;
    pub mod token;
    mod internal;   // private submodule
}
```

### 10.2 Imports

```sploosh
use std::collections::Map;
use crate::auth::token::verify;
use crate::models::{User, Role, Permission};
```

### 10.3 Visibility

- `pub` — visible outside the module
- (no modifier) — private to the module

Two levels only. No `protected`, no `internal`, no `pub(crate)`. One way to do it.

---

## 11. Web3 Extensions

### 11.1 On-Chain / Off-Chain Separation

```sploosh
onchain mod token {
    storage {
        balances: Map<Address, u256>,
        total_supply: u256,
        owner: Address,
    }

    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError> {
        let sender = ctx::caller();
        let balance = storage::get(&self.balances, sender)?;

        if balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        storage::set(&mut self.balances, sender, balance - amount);
        storage::set(&mut self.balances, to,
            storage::get(&self.balances, to)? + amount);

        emit Transfer { from: sender, to, amount };
        Ok(())
    }
}
```

### 11.2 The `ctx` Module (On-Chain Context)

All on-chain functions have access to `ctx`, which provides blockchain execution context.

**Universal (all targets):**

| Function | Return Type | Purpose |
|---|---|---|
| `ctx::caller()` | `Address` | Address that invoked this function |
| `ctx::self_address()` | `Address` | This contract's address |
| `ctx::timestamp()` | `u256` | Current block timestamp (seconds) |
| `ctx::block_number()` | `u256` | Current block number / slot |

**EVM-specific:**

| Function | Return Type | Purpose |
|---|---|---|
| `ctx::value()` | `u256` | ETH sent with the call (in wei) |
| `ctx::gas_remaining()` | `u256` | Remaining gas |
| `ctx::chain_id()` | `u256` | EVM chain ID |

**SVM-specific (Solana):**

| Function | Return Type | Purpose |
|---|---|---|
| `ctx::lamports()` | `u64` | SOL sent with the instruction |
| `ctx::program_id()` | `Address` | This program's address |
| `ctx::signer()` | `Address` | Transaction signer |

### 11.3 Payable Functions and Reentrancy

Functions that receive native tokens must be annotated with `@payable`.
Calling `ctx::value()` in a non-`@payable` function is a compile-time error.

```sploosh
onchain mod vault {
    storage {
        balances: Map<Address, u256>,
    }

    @payable
    pub fn deposit() -> Result<(), VaultError> {
        let sender = ctx::caller();
        let amount = ctx::value();

        let current = storage::get(&self.balances, sender).unwrap_or(0);
        storage::set(&mut self.balances, sender, current + amount);

        emit Deposit { sender, amount };
        Ok(())
    }
}
```

**Reentrancy:** On-chain functions are **non-reentrant by default**. A function
cannot be called again while it is already executing.

### 11.4 Cross-Contract Calls

```sploosh
onchain mod lending {
    pub fn borrow(token_addr: Address, amount: u256) -> Result<(), LendError> {
        let sender = ctx::caller();

        let balance = chain::call(
            token_addr,
            token::balance_of,
            sender
        )?;

        if balance < amount * 2 {
            return Err(LendError::InsufficientCollateral);
        }

        chain::call(
            token_addr,
            token::transfer_from,
            (sender, ctx::self_address(), amount)
        )?;

        Ok(())
    }
}
```

### 11.5 Events

```sploosh
onchain enum Event {
    Transfer { from: Address, to: Address, amount: u256 },
    Approval { owner: Address, spender: Address, amount: u256 },
    Deposit { sender: Address, amount: u256 },
}
```

### 11.6 Off-Chain Calling On-Chain

```sploosh
offchain fn check_balance(user: Address) -> Result<u256, AppError> {
    let contract = Contract::connect("0x1234...")?;
    let balance = contract.call(token::balance_of, user).await?;
    Ok(balance)
}
```

### 11.7 Compile Targets

```
sploosh build --target native       # LLVM → native binary
sploosh build --target wasm          # LLVM → WebAssembly
sploosh build --target evm           # on-chain → EVM bytecode
sploosh build --target svm           # on-chain → Solana SBF
```

---

## 12. Attributes & Compiler Directives

### 12.1 Standard Attributes

| Attribute | Purpose |
|---|---|
| `@test` | Mark a function as a test case |
| `@derive(...)` | Auto-generate trait impls |
| `@error` | Auto-generate `From`, `Display`, `Error` for error enums |
| `@inline` | Hint to inline a function |
| `@payable` | On-chain: function accepts native tokens |
| `@reentrant` | On-chain: opt-in to reentrancy (discouraged) |
| `@supervisor(...)` | Mark an actor as a supervisor |

### 12.2 Derive Macros

| Derive | Generates |
|---|---|
| `Debug` | `Debug` trait (for `{:?}`) |
| `Clone` | `Clone` trait (deep copy) |
| `Copy` | `Copy` trait (bitwise copy, requires `Clone`) |
| `Eq` | `Eq` trait (structural equality) |
| `Hash` | `Hash` trait |
| `Serialize` | `Serialize` trait |
| `Deserialize` | `Deserialize` trait |
| `Ord` | `Ord` trait (total ordering) |

All derive macros work on structs and enums with mixed variant types.

### 12.3 Compiler Directives and Conditional Compilation

```sploosh
#[target(evm)]         // Compile only for EVM target
#[target(native)]      // Compile only for native target
#[gas_limit(50000)]    // On-chain gas budget
#[cfg(test)]           // Include only in test builds
#[cfg(debug)]          // Include only in debug builds
```

**Available `cfg` flags:**

| Flag | True when |
|---|---|
| `#[cfg(test)]` | Running `sploosh test` |
| `#[cfg(debug)]` | Building without `--release` |
| `#[cfg(release)]` | Building with `--release` |
| `#[cfg(target = "native")]` | `--target native` |
| `#[cfg(target = "wasm")]` | `--target wasm` |
| `#[cfg(target = "evm")]` | `--target evm` |
| `#[cfg(target = "svm")]` | `--target svm` |
| `#[cfg(feature = "name")]` | Feature enabled in `sploosh.toml` |

**On-chain stdlib restrictions:** `onchain` modules automatically cannot use I/O-bound
standard library modules. The following are compile-time errors inside `onchain`:

- `std::fs` — no filesystem
- `std::net` — no networking
- `std::io` — no stdin/stdout
- `std::db` — no database
- `std::web` — no HTTP server
- `std::env` — no environment

Available inside `onchain`: `std::math`, `std::crypto`, `std::chain`, `std::collections`,
and all core types.

**Portable code pattern:**

```sploosh
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    // This works on all targets — std::crypto is universal
    std::crypto::sha256(data)
}

#[cfg(target = "native")]
pub fn save_to_disk(data: &[u8]) -> Result<(), AppError> {
    fs::write("output.bin", data)?;
    Ok(())
}

#[cfg(target = "wasm")]
pub fn save_to_disk(data: &[u8]) -> Result<(), AppError> {
    // WASM: use browser API or return unsupported
    Err(AppError::Unsupported { feature: "filesystem".into() })
}
```

**Feature flags in sploosh.toml:**

```toml
[features]
default = ["json"]
json = []
postgres = ["sploosh_db"]
```

---

## 13. Standard Library (Core)

### 13.1 Prelude (auto-imported)

```
Option, Some, None
Result, Ok, Err
String, Vec, Map, Set
print, format, assert
Display, Debug, Clone, Copy, Eq, Hash, Ord
Fn, FnMut, FnOnce
Iter, FromIter
Send, Sync
Handle
```

### 13.2 Core Modules

| Module         | Purpose                                  | Targets |
|----------------|------------------------------------------|---------|
| `std::io`      | File I/O, stdin/stdout                   | native, wasm |
| `std::net`     | TCP/UDP, HTTP client                     | native, wasm |
| `std::json`    | JSON parse/serialize                     | all |
| `std::crypto`  | Hashing, signing, key generation         | all |
| `std::time`    | Timestamps, durations, timers            | native, wasm |
| `std::math`    | Numeric operations, constants            | all |
| `std::collections` | Advanced data structures             | all |
| `std::fs`      | Filesystem operations                    | native |
| `std::env`     | Environment variables, CLI args          | native |
| `std::log`     | Structured logging                       | native, wasm |
| `std::test`    | Test framework, assertions               | native (test) |
| `std::web`     | HTTP server, routing, middleware         | native, wasm |
| `std::db`      | Database connection, query builder       | native |
| `std::chain`   | Web3 utilities, ABI encoding, addresses  | all |

---

## 14. File Structure

```
project/
├── sploosh.toml          # Project manifest
├── src/
│   ├── main.sp         # Entry point
│   ├── lib.sp          # Library root
│   ├── models/
│   │   ├── mod.sp
│   │   ├── user.sp
│   │   └── role.sp
│   ├── handlers/
│   │   ├── mod.sp
│   │   └── api.sp
│   └── contracts/
│       ├── mod.sp
│       └── token.sp    # on-chain code
├── tests/
│   └── integration.sp
└── deploy/
    └── evm.toml        # Chain deployment config
```

### 14.1 Project Manifest

```toml
[project]
name = "my-app"
version = "0.1.0"
edition = "2026"

[dependencies]
sploosh_web = "0.3"
sploosh_db = "0.2"

[features]
default = ["json"]
json = []
postgres = ["sploosh_db"]

[targets]
default = "native"
contracts = ["evm", "svm"]
```

---

## 15. Complete Example: REST API with On-Chain Integration

```sploosh
use std::web::{Server, Router, Request, Response, Status};
use std::json;
use std::db::Pool;
use crate::contracts::token;
use crate::models::User;

struct AppState {
    db: Pool,
    contract: Contract,
}

fn main() -> Result<(), AppError> {
    let state = AppState {
        db: Pool::connect("postgres://localhost/myapp")?,
        contract: Contract::connect("0xABC123...")?,
    };

    let router = Router::new()
        |> route("GET", "/users/:id", get_user)
        |> route("POST", "/transfer", transfer_tokens)
        |> middleware(auth::require_token);

    Server::bind("0.0.0.0:8080")
        |> serve(router, state)?;

    Ok(())
}

async fn get_user(req: &Request, state: &AppState) -> Result<Response, AppError> {
    let id: u64 = req.param("id")?.parse()?;

    let user = state.db
        .query("SELECT * FROM users WHERE id = $1", &[id])
        .await?
        |> map(User::from)
        |> first
        |> ok_or(AppError::NotFound { resource: "user".into() })?;

    let balance = state.contract
        .call(token::balance_of, user.wallet)
        .await?;

    let body = json::to_string(&UserResponse {
        user,
        token_balance: balance,
    })?;

    Ok(Response::new(Status::Ok, body))
}

async fn transfer_tokens(req: &Request, state: &AppState) -> Result<Response, AppError> {
    let body: TransferRequest = json::from_reader(req.body())?;

    let tx = state.contract
        .send(token::transfer, body.to, body.amount)
        .await?;

    Ok(Response::new(Status::Ok, json::to_string(&tx)?))
}
```

---

## 16. Grammar (EBNF Summary)

```ebnf
program        = { item } ;
item           = fn_def | struct_def | enum_def | trait_def
               | impl_block | mod_def | use_stmt | actor_def
               | onchain_mod | const_def | type_alias ;

fn_def         = [ "pub" ] [ "async" ] "fn" IDENT [ generics ] "(" params ")"
                 [ "->" type ] block ;
params         = [ param { "," param } ] ;
param          = IDENT ":" type ;

struct_def     = [ attrs ] [ "pub" ] "struct" IDENT [ generics ] "{" fields "}" ;
fields         = field { "," field } [ "," ] ;
field          = [ "pub" ] IDENT ":" type ;

enum_def       = [ attrs ] [ "pub" ] "enum" IDENT [ generics ] "{" variants "}" ;
variants       = variant { "," variant } [ "," ] ;
variant        = IDENT [ "(" types ")" | "{" fields "}" ] ;

trait_def      = [ "pub" ] "trait" IDENT [ generics ] [ ":" bounds ] "{" { trait_item } "}" ;
trait_item     = fn_sig [ block ] ";" ;

impl_block     = "impl" [ generics ] [ trait "for" ] type "{" { fn_def } "}" ;

actor_def      = [ attrs ] "actor" IDENT [ generics ] "{" { actor_item } "}" ;
actor_item     = field_def | fn_def ;

mod_def        = [ "pub" ] "mod" IDENT ( ";" | "{" { item } "}" ) ;
use_stmt       = "use" path [ "::" "{" idents "}" ] ";" ;

onchain_mod    = "onchain" "mod" IDENT "{" { onchain_item } "}" ;
onchain_item   = storage_block | fn_def | event_def ;
storage_block  = "storage" "{" fields "}" ;

type           = prim_type | IDENT [ generics ] | "&" [ "mut" ] type
               | "[" type ";" EXPR "]" | "[" type "]"
               | "(" [ type { "," type } ] ")" | "fn" "(" types ")" "->" type
               | "dyn" IDENT [ generics ] | "Box" "<" "dyn" IDENT ">" ;
generics       = "<" type_params ">" ;
type_params    = type_param { "," type_param } ;
type_param     = IDENT [ ":" bounds ] ;
bounds         = bound { "+" bound } ;
bound          = IDENT [ generics ] | lifetime ;

block          = "{" { statement } [ expr ] "}" ;
statement      = let_stmt | expr_stmt | return_stmt ;
let_stmt       = "let" [ "mut" ] pattern [ ":" type ] "=" expr ";" ;
const_def      = [ "pub" ] "const" IDENT ":" type "=" expr ";" ;
return_stmt    = "return" [ expr ] ";" ;
expr_stmt      = expr ";" ;

expr           = literal | IDENT | path_expr
               | expr "." IDENT | expr "(" args ")"  | expr "[" expr "]"
               | expr BINOP expr | UNOP expr | expr "?"
               | if_expr | if_let_expr | match_expr | block | closure
               | expr "|>" expr
               | "spawn" expr | "send" expr | "recv" expr
               | expr ".await"
               | "for" pattern "in" expr block
               | "while" expr block | while_let_expr | "loop" block ;

if_expr        = "if" expr block [ "else" ( if_expr | if_let_expr | block ) ] ;
if_let_expr    = "if" "let" pattern "=" expr block [ "else" block ] ;
while_let_expr = "while" "let" pattern "=" expr block ;
match_expr     = "match" expr "{" { match_arm } "}" ;
match_arm      = pattern [ "if" expr ] "=>" ( expr "," | block ) ;
closure        = [ "move" ] "|" params "|" ( expr | block ) ;

pattern        = "_" | literal | IDENT | [ "ref" ] IDENT
               | IDENT "(" patterns ")" | IDENT "{" field_pats [ ".." ] "}"
               | "(" patterns ")" | pattern "|" pattern ;

literal        = INT_LIT [ type_suffix ] | FLOAT_LIT [ type_suffix ]
               | STRING_LIT | CHAR_LIT
               | "true" | "false" | "none" ;

attrs          = { "@" IDENT [ "(" attr_args ")" ] } ;
```

---

## 17. Design Decisions Log

| Decision | Rationale |
|---|---|
| Braces `{}` not indentation | 8 of top 10 languages use braces. Higher LLM accuracy. |
| `fn` not `func`/`function`/`def` | Shortest. Rust-trained models produce it most reliably. |
| `let` not `var`/`auto`/`val` | Universal across Rust, JS, TS, Swift. |
| `match` with `=>` arms | Rust pattern. Exhaustive by default. Deeply trained. |
| `\|>` pipe operator | Elixir/F# pattern. Eliminates deep nesting. |
| `?` error propagation | Rust pattern. Single token. Explicit. Universally understood. |
| `expr \|> f?` = `f(expr)?` | Clear precedence rule. ? always applies to the pipe result. |
| Pipe fills first argument | `x \|> f(a)` = `f(x, a)`. No placeholder syntax. Use closures for other positions. |
| `actor` keyword | Self-documenting English word. Clear semantic meaning. |
| `Handle<T>` for actor refs | Familiar generic syntax. Clear it's a reference, not the actor. |
| Actor params must be owned | No `&T` in actor pub methods. Messages are async — borrows would dangle. |
| `move` for closure capture | Directly from Rust. Deeply trained. Explicit ownership transfer. |
| `@error` derive for error enums | Reduces boilerplate for the most common error pattern. |
| `.context()` for error wrapping | Familiar from Rust's anyhow. Explicit chain, no hidden magic. |
| `Iter` trait with lazy adaptors | Matches Rust. Prevents accidental O(n) allocations. |
| `if let` / `while let` | From Rust. Deeply trained. Concise single-pattern matching. |
| Destructuring in `let` | Tuple, struct, nested patterns in bindings. Familiar from Rust/JS/Python. |
| `dyn Trait` for dynamic dispatch | From Rust. Explicit runtime cost. Clear when you pay for indirection. |
| Default `i64` / `f64` | Matches JavaScript semantics (f64) and avoids i32 overflow surprises. |
| No `static mut` | All mutable state lives in actors. Eliminates data races by construction. |
| `onchain` restricts stdlib | Compile-time enforcement. Can't accidentally use `fs::read` in a smart contract. |
| `format()` not f-strings | One way to format strings. Compiler intrinsic, not a macro. |
| No `+` for string concat | No operator overloading means `+` is always arithmetic. Use `format` or `push_str`. |
| No `null`/`nil` | Eliminates the billion-dollar mistake. `Option<T>` is explicit. |
| No exceptions | `Result<T, E>` forces handling at every call site. |
| No operator overloading | `+` always means addition. No hidden behavior. |
| ASCII-only syntax | Max tokenizer efficiency. Zero multi-byte operator ambiguity. |
| `.sp` file extension | Short, unique, no conflicts with existing languages. |
| Explicit lifetimes (no elision) | LLMs generate more correct code when lifetimes are visible. |
| Two visibility levels only | `pub` or private. No decision fatigue for the model. |

---

## Appendix A: Token Budget Analysis

A typical 40-line Sploosh function uses approximately:

| Component | Tokens (est.) |
|---|---|
| Keywords (`fn`, `let`, `match`, etc.) | ~15 |
| Identifiers (names, types) | ~40 |
| Operators (`->`, `?`, `\|>`, etc.) | ~12 |
| Delimiters (`{`, `}`, `(`, `)`, `,`) | ~25 |
| Literals (strings, numbers) | ~10 |
| **Total** | **~102 tokens** |

Comparable Python: ~130 tokens. Comparable Rust: ~115 tokens.

---

## Appendix B: Compilation Pipeline

```
Source (.sp)
    │
    ├─ Lexer ──► Token Stream
    │
    ├─ Parser ──► AST
    │
    ├─ Type Checker ──► Typed AST
    │
    ├─ Ownership/Borrow Checker
    │
    ├─ IR Lowering ──► Sploosh IR
    │
    ├─► LLVM Backend ──► Native Binary / WASM
    │
    ├─► EVM Backend ──► Solidity Yul ──► EVM Bytecode
    │
    └─► SVM Backend ──► Solana SBF
```

---

## Appendix C: Comparison with Existing Languages

| Feature | Sploosh | Rust | Elixir | Solidity | TypeScript |
|---|---|---|---|---|---|
| Memory safety | Ownership | Ownership | GC (BEAM) | EVM stack | GC (V8) |
| Error handling | Result + ? | Result + ? | {:ok}/{:error} | require/revert | try/catch |
| Concurrency | Actors | async/threads | Actors (BEAM) | N/A | async/await |
| Pattern matching | match (exhaustive) | match | case (exhaustive) | N/A | switch (non-exhaustive) |
| Pipe operator | \|> | None | \|> | None | None (proposal) |
| Smart contracts | onchain blocks | Via ink! | None | Native | None |
| Null safety | No null | No null | nil exists | No null | Optional chaining |
| Compile target | LLVM+EVM+SVM | LLVM | BEAM | EVM | V8/Bun |

---

## Appendix D: Amendment History

| Version | Changes |
|---|---|
| v0.1.0 | Initial draft. Core syntax, types, ownership, actors, web3. |
| v0.2.0 | Added: closure capture semantics (§4.5), type unification & pattern binding rules (§3.7), `Handle<T>` actor handle types (§8.2), generic actors (§8.3), pipe + error propagation rules (§5.5), format specifiers (§9), self-matching in impls (§5.2), `ctx` API surface (§11.2), `@payable` and reentrancy (§11.3), cross-contract calls (§11.4), iterator protocol and collection methods (§7), `@error` derive macro (§6.3), error context/chaining (§6.4), derive macro reference (§12.2). EBNF updated for `move` closures, `ref` patterns, generic actors. Keyword count 37→38 (`move`). |
| v0.3.0 | Added: pipe argument position rules (§5.6), actor message ownership — no references in pub methods (§8.2), type inference rules with default i64/f64 (§3.8), dynamic dispatch / `dyn Trait` / `Box<dyn Trait>` with object safety rules (§3.9), actor failure and recovery / `ActorError` (§8.7), constant evaluation rules / no `static` (§4.6), string methods on `str`/`String` / no `+` concat (§9.5), conditional compilation `cfg` flags and onchain stdlib restrictions (§12.3), supertrait syntax and struct generic bounds (§3.5-3.6), destructuring in `let` bindings (§5.3), `if let` and `while let` (§5.4), destructuring in `for` loops (§5.5). EBNF updated for `if_let_expr`, `while_let_expr`, `dyn` types, `..` rest pattern, numeric literal suffixes. Stdlib table now includes target availability. Feature flags in sploosh.toml. |

---

*Working title: Sploosh. Name subject to change.*
*This spec is a living document. v0.3.0-draft — April 2026.*
