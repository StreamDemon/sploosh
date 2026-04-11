# SPLOOSH Language Specification v0.4.3-draft

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

### 2.3 Keywords (40 total)

**Declarations:**
`fn` `let` `const` `type` `struct` `enum` `trait` `impl` `mod` `use` `pub` `extern`

**Control Flow:**
`if` `else` `match` `for` `in` `while` `loop` `break` `continue` `return`

**Types & Values:**
`self` `Self` `true` `false` `none` `as`

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
| 12   | `as`         | Numeric cast               | Left  |
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

### 2.6 Literals

#### 2.6.1 Numeric Literals

**Integer literals** may be written in four bases:

```sploosh
let dec = 42;          // decimal
let hex = 0xFF;        // hexadecimal
let oct = 0o755;       // octal
let bin = 0b1010_0101; // binary
```

**Underscore separators** are permitted anywhere between digits for readability:

```sploosh
let million   = 1_000_000;
let max_u64   = 0xFFFF_FFFF_FFFF_FFFF;
let addr_mask = 0b1111_1111_0000_0000;
```

Underscores must appear between two digits — leading, trailing, or consecutive underscores
in a numeric body are a compile error (e.g., `_1`, `1_`, `1__2` are all invalid).

**Type suffixes** bind a literal to a specific numeric type:

```sploosh
let a = 42u32;     // u32
let b = 0xFFu8;    // u8
let c = 3.14f32;   // f32
let d = 0u256;     // u256
```

Suffixes: `i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 u256 f32 f64`. An integer literal
with a float suffix (e.g., `42f64`) is equivalent to `42.0f64`.

**Literal overflow is a compile error.** If a literal value does not fit in its declared
or inferred type, the compiler rejects it at parse time:

```sploosh
let a: u8  = 256;     // compile error: 256 does not fit in u8
let b      = 256u8;   // compile error: 256 does not fit in u8
let c: i32 = 0x8000_0000i32;  // compile error: out of i32 range
```

**Float literals** require either a decimal point or an exponent (or both):

```sploosh
let pi    = 3.14;
let big   = 1e10;        // 1.0 × 10^10
let small = 1.5e-3;      // 1.5 × 10^-3
let typed = 2.5e2f32;    // 250.0 as f32
```

A bare integer sequence is never parsed as a float — `42` is `i64` (or inferred),
`42.0` or `42f64` is `f64`. The forms `42.` and `.5` are rejected to avoid ambiguity
with method call syntax and range operators.

#### 2.6.2 String Literals

String literals are double-quoted UTF-8 text:

```sploosh
let greeting = "Hello, world!";
let multi    = "line one\nline two";
let unicode  = "café \u{1F600}";
```

**Escape sequences** recognized inside string and character literals:

| Escape | Meaning |
|---|---|
| `\n` | newline (U+000A) |
| `\r` | carriage return (U+000D) |
| `\t` | horizontal tab (U+0009) |
| `\\` | backslash |
| `\"` | double quote |
| `\'` | single quote |
| `\0` | null (U+0000) |
| `\xNN` | ASCII byte escape, `NN` is two hex digits, value must be `0x00`–`0x7F` |
| `\u{H...}` | Unicode scalar value, 1–6 hex digits, must be a valid scalar (no surrogates `D800`–`DFFF`) |

Any other backslash sequence is a compile error. Raw string literals (`r"..."`,
`r#"..."#`) are deferred to a future version — use escape sequences in v0.4.

String literals may span multiple source lines; a literal newline inside the string
becomes `\n` in the value. To continue a long string without embedding a newline,
end each line with `\` immediately before the newline — the backslash, the newline,
and any leading whitespace on the next line are all elided.

#### 2.6.3 Character Literals

Character literals are single-quoted and represent a single Unicode scalar value:

```sploosh
let a      = 'a';
let tab    = '\t';
let quote  = '\'';
let emoji  = '\u{1F600}';
```

The body is exactly one Unicode scalar value written either directly (`'a'`, `'é'`)
or via an escape sequence from the table above. A character literal cannot contain
a raw single quote, a raw backslash, a raw newline, or a surrogate code point.
`char` is the type of a Unicode scalar value, not a byte or a UTF-16 code unit.

### 2.7 Identifiers

An identifier starts with an ASCII letter or underscore and continues with any number
of ASCII letters, digits, or underscores:

```
IDENT = [A-Za-z_] [A-Za-z0-9_]*
```

Identifiers are ASCII-only (per §2.1). Unicode identifiers are not supported — the
design goal is zero tokenizer ambiguity and maximum LLM accuracy.

**Keyword priority.** If an identifier matches any keyword in §2.3, it is tokenized
as that keyword, not as an identifier. Raw identifiers (`r#keyword`) are deferred to
a future version.

**The wildcard binding.** A single underscore `_` is not a regular identifier — it is
a wildcard binding that discards its value and does not introduce a name. Each `_` in
a scope is fresh and cannot be referenced:

```sploosh
let _ = compute();          // discard the result
let (x, _, z) = triple();   // keep first and third, discard middle
```

Identifiers that begin with `_` followed by at least one other character (e.g.,
`_unused`, `_tmp`) are regular identifiers — they introduce a binding and can be
referenced, but the compiler suppresses the usual "unused variable" warning as a
convention.

**Length.** Identifiers have no formal length limit. Implementations should accept at
least 1024 characters. Tooling may warn on identifiers longer than 64 characters.

---

## 3. Type System

### 3.1 Primitive Types

```
i8  i16  i32  i64  i128    // Signed integers
u8  u16  u32  u64  u128    // Unsigned integers
u256                        // 256-bit unsigned integer (checked arithmetic always)
f32  f64                    // Floating point
bool                        // true | false
char                        // Unicode scalar value
str                         // String slice (immutable)
String                      // Owned, growable string
Address                     // Blockchain address (32 bytes, not an integer)
()                          // Unit type (void equivalent)
```

**`u256`**: 256-bit unsigned integer. Available on all targets. Literal suffix: `0u256`,
hex: `0xFF00u256`. Always uses checked arithmetic regardless of build mode.

**`Address`**: 32-byte blockchain address. No arithmetic operations. Supports `==`, `!=`,
`Display`, `Debug`, `Clone`, `Copy`, `Hash`, `Eq`, `Ord`. Construct via `Address::from_hex("0x...")`.

**In-memory representation:** always exactly 32 bytes, stored big-endian. This
representation is identical on every target (native, wasm, EVM, SVM) so values can be
passed across module and target boundaries without conversion.

**EVM serialization:** when an `Address` is written to EVM storage, used as an event
topic, or passed across an EVM ABI boundary, the 32-byte value is **left-padded** to
match Solidity's convention — the low 20 bytes are the address proper and the high 12
bytes are zeros. Sploosh rejects any `Address` value whose high 12 bytes are non-zero
when serializing to EVM (this can only happen if the value was constructed from an
SVM-specific 32-byte public key).

**SVM serialization:** the full 32 bytes are used as-is (Solana public keys are
32 bytes natively). No padding, no truncation.

No-padding behavior is observable only at the serialization boundary; user code
always sees a uniform 32-byte value.

### 3.2 Compound Types

```
[T; N]              // Fixed-size array
[T]                 // Slice
Vec<T>              // Growable list
Map<K, V>           // Hash map
Set<T>              // Hash set
Box<T>              // Heap-allocated owned value
(T, U, V)           // Tuple
Option<T>           // Some(T) | None
Result<T, E>        // Ok(T) | Err(E)
Channel<T>          // Bounded MPSC channel (via Channel::new(cap))
Sender<T>           // Channel send endpoint (Clone + Send)
Receiver<T>         // Channel receive endpoint (not Clone)
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
trait Encode {
    fn to_bytes(&self) -> Vec<u8>;
    fn size_hint(&self) -> u64 { 0 }   // Default implementation
}

impl Encode for User {
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

**Associated types:** Traits can declare associated types that implementors must specify:

```sploosh
trait Iter {
    type Item;    // Associated type — implementors specify the concrete type
    fn next(&mut self) -> Option<Self::Item>;
}

impl Iter for NumberRange {
    type Item = i64;    // Concrete type for this implementation
    fn next(&mut self) -> Option<i64> { /* ... */ }
}
```

Associated type rules:
- Declared with `type Name;` inside a trait definition.
- Implementors provide `type Name = ConcreteType;` in their `impl` block.
- Trait bounds can constrain associated types: `T: Iter<Item = String>`.

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

### 3.10 Standard Traits

Sploosh defines the following standard traits. All are in the prelude.

**Marker traits** (no methods, auto-implemented):

| Trait | Purpose | Rules |
|---|---|---|
| `Copy` | Bitwise duplicate on assignment | Requires `Clone`. Mutually exclusive with `Drop`. |
| `Send` | Can be transferred between threads/actors | Auto-implemented for types with all `Send` fields. |
| `Sync` | Safe to share via `&T` across threads | Auto-implemented for types with all `Sync` fields. |

**Core traits** (derivable with `@derive`):

| Trait | Method | Purpose |
|---|---|---|
| `Clone` | `fn clone(&self) -> Self` | Deep copy |
| `Debug` | `fn fmt_debug(&self) -> String` | Debug representation for `{:?}` |
| `Eq` | `fn eq(&self, other: &Self) -> bool` | Structural equality (also generates `!=`) |
| `Ord` | `fn cmp(&self, other: &Self) -> Ordering` | Total ordering. Requires `Eq`. Enables `<` `>` `<=` `>=` |
| `Hash` | `fn hash(&self) -> u64` | Hash value. Required for `Map` keys and `Set` elements |
| `Serialize` | `fn serialize(&self) -> Vec<u8>` | Binary serialization |
| `Deserialize` | `fn deserialize(bytes: &[u8]) -> Result<Self, DeserializeError>` | Binary deserialization |

**Conversion traits:**

| Trait | Method | Purpose |
|---|---|---|
| `From<T>` | `fn from(value: T) -> Self` | Infallible conversion |
| `Into<T>` | (auto from `From`) | `val.into()` calls `T::from(val)` |
| `TryFrom<T>` | `fn try_from(value: T) -> Result<Self, Self::Error>` | Fallible conversion |
| `TryInto<T>` | (auto from `TryFrom`) | Fallible `.try_into()` |
| `Display` | `fn to_string(&self) -> String` | Human-readable format for `{}` |

**Error and cleanup traits:**

| Trait | Method | Purpose |
|---|---|---|
| `Error` | `fn message(&self) -> String` | Supertrait: `Error: Display`. Base for all error types |
| `Drop` | `fn drop(&mut self)` | Deterministic cleanup when value goes out of scope |

`Drop` rules:
- Drop order within a scope: reverse of declaration order.
- Struct fields drop in declaration order.
- `Drop` and `Copy` are mutually exclusive — a type cannot implement both.
- Implement `Drop` for custom cleanup (file handles, network connections).

**Closure traits** (already defined in §4.5):
`Fn(Args) -> Ret`, `FnMut(Args) -> Ret`, `FnOnce(Args) -> Ret`

**Iterator traits:**
- `Iter` — defined in §7.1.
- `FromIter` — `fn from_iter(iter: impl Iter<Item = T>) -> Self`. Used by `collect()`.

### 3.11 Numeric Casting with `as`

The `as` keyword performs numeric type conversions only.

```sploosh
let x: i32 = 42;
let y: i64 = x as i64;         // widening — always safe
let z: u8 = 256u16 as u8;      // narrowing — truncates (result: 0)
let w: i64 = 3.7f64 as i64;    // float to int — truncates toward zero (result: 3)
let f: f64 = 42i64 as f64;     // int to float
```

**Rules:**
- `as` works between integer types, between float types, and between integer and float.
- Narrowing conversions truncate (same as two's complement for integers).
- Float-to-int truncates toward zero. Out-of-range finite values saturate to the target type's bounds.
- **NaN and infinity.** When casting a floating-point value to an integer type via `as`:
  - `NaN` → `0`
  - positive infinity → the target type's `MAX`
  - negative infinity → the target type's `MIN` (for signed types) or `0` (for unsigned types)

  This matches WebAssembly's `trunc_sat` semantics and avoids the undefined behavior
  that plagues C and early Rust float-to-int casts. The behavior is identical on every
  target — no implementation-defined drift.
- `as` does NOT work for non-numeric conversions. Use `From`/`Into` for those.
- `as` is NOT a reference coercion or type alias. It is purely numeric.

```sploosh
let nan     = f64::NAN as i32;            // 0
let pos_inf = f64::INFINITY as u32;       // u32::MAX
let neg_inf = f64::NEG_INFINITY as i32;   // i32::MIN
let neg_u   = f64::NEG_INFINITY as u32;   // 0
let huge    = 1e20f64 as i32;             // i32::MAX (saturates)
```

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

### 4.4 Heap Allocation with Box\<T\>

`Box<T>` allocates a value on the heap with single-owner semantics:

```sploosh
let boxed: Box<i64> = Box::new(42);    // heap-allocated
let val: i64 = *boxed;                  // deref to inner value
```

- `Box<T>` is dropped when the owner goes out of scope (calling `Drop` if implemented).
- `Box<T>` is `Send` if `T: Send`. `Clone` if `T: Clone`.
- Primary use: trait objects (`Box<dyn Trait>`), large values, recursive types.

**No `Rc<T>` or `Arc<T>` in Sploosh.** Use `Handle<T>` for sharing state across actors.
Use `Map<Id, T>` with integer IDs for graph-like structures within a single actor.

### 4.5 Lifetimes

Lifetime annotations specify how long references are valid. They are required when
a function **returns a reference** and has multiple reference parameters.

**Rules:**
1. If a function does not return a reference, no lifetime annotations are needed.
2. **Single-source rule:** When a function has exactly one reference parameter
   (including `&self`/`&mut self`) and returns a reference, the output lifetime
   equals the input lifetime. No annotation needed.
3. When multiple reference parameters exist and a reference is returned, explicit
   lifetime annotations are required.

```sploosh
// No reference return — no annotations needed
fn greet(name: &str) -> String { format("Hello, {}", name) }

// Single source — elided (output lifetime = input lifetime)
fn first_word(s: &str) -> &str { /* ... */ }

// &self is the single source — elided
fn name(&self) -> &str { &self.name }

// Multiple sources — explicit lifetime required
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}

// One source matters, others ignored — explicit required
fn pick<'a>(a: &'a str, b: &str) -> &'a str { a }
```

### 4.6 Closures and Capture Semantics

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

### 4.7 Constants

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

### 4.8 Integer Overflow

All integer arithmetic (`+`, `-`, `*`) is **checked by default** in all build modes
and on all targets. Overflow causes:
- **In actors**: the actor dies (supervisor restarts if configured).
- **In non-actor code**: the program aborts with an error.
- **On-chain**: the transaction reverts.

**Explicit wrapping and saturating operations** are provided as methods on all integer types
for cases where wrapping is intentional (cryptography, hashing, bit manipulation):

```sploosh
let a: u8 = 200;
let b: u8 = 100;

// These would abort (checked overflow):
// let c = a + b;                        // 300 > 255 — overflow!

// Intentional wrapping:
let c = a.wrapping_add(b);               // 44 (two's complement)

// Saturating:
let d = a.saturating_add(b);             // 255 (clamped at max)

// Checked (returns Option):
let e: Option<u8> = a.checked_add(b);    // None
```

Available on all integer types (`i8`..`i128`, `u8`..`u128`, `u256`):
`wrapping_add`, `wrapping_sub`, `wrapping_mul`,
`saturating_add`, `saturating_sub`,
`checked_add`, `checked_sub`, `checked_mul`.

The `@overflow(wrapping)` attribute opts a function into wrapping arithmetic:

```sploosh
@overflow(wrapping)
fn hash_combine(a: u64, b: u64) -> u64 {
    a * 6364136223846793005 + b    // wrapping is intentional
}
```

`@overflow(wrapping)` is a **compile error inside `onchain` modules**. On-chain
arithmetic is always checked — no exceptions.

### 4.9 Foreign Function Interface (extern)

Sploosh has no `unsafe` keyword. Foreign functions are declared with `extern "C"` blocks,
and the compiler generates safe wrappers automatically:

```sploosh
extern "C" {
    fn c_open(path: &str, flags: i32) -> Result<i32, FfiError>;
    fn c_close(fd: i32) -> Result<(), FfiError>;
}
```

**FFI rules:**
- `extern "C"` blocks are only allowed at module top level (not inside functions or actors).
- The compiler generates safe wrapper code. No raw pointer types are exposed to user code.
- Null pointers from C are converted to `Option::None`. Non-null becomes `Some(&T)`.
- C functions that can fail are wrapped to return `Result<T, FfiError>`.
- `extern` blocks are not available inside `onchain` modules (compile error).
- There are no raw pointer types (`*const T`, `*mut T`) in the language.
- **Handler-safe FFI.** `extern "C"` blocks may be marked `async` —
  `extern "C" async { fn native_fetch(...) -> Result<..., FfiError>; }` — in
  which case the compiler emits an awaitable wrapper that offloads the
  underlying synchronous C call to the runtime's blocking pool. Only
  `extern "C" async` functions may be called (directly or transitively) from
  inside an actor message handler; calling a plain (synchronous) `extern "C"`
  function from a handler is a compile error. See §8.11a for the handler rule.

### 4.10 Floating-Point and Math Operations

Sploosh exposes the full IEEE 754 math surface on `f32` and `f64` through **method syntax**,
matching the convention established by integer methods in §4.8. All math methods are
**compiler intrinsics** — the compiler lowers them directly to LLVM intrinsics
(`llvm.sin.f64`, `llvm.sqrt.f64`, `llvm.fma.f64`, etc.) rather than through opaque libm
calls. This is a load-bearing design choice: intrinsic lowering is what enables compile-time
constant folding (`(0.0f64).sin()` → `0.0` during codegen), auto-vectorization of math
inside loops, and fusion of adjacent `.sin()` + `.cos()` calls into `llvm.sincos`.

**Correctness contract.** In the default mode (no `@fast_math`), math methods produce
results within 1 ULP of the correctly-rounded IEEE 754 value. Implementations may lower
to LLVM libc libm for functions where LLVM intrinsics do not guarantee correct rounding
(`sin`, `cos`, `log`, `exp`, `pow`). `sqrt` and `fma` are correctly rounded on all supported
targets. `@fast_math(afn)` relaxes the 1-ULP bound to implementation-defined accuracy.

**Method categories on `f32` and `f64`** (full signatures in `stdlib/math.md`):

- **Classification:** `is_nan`, `is_finite`, `is_infinite`, `is_normal`, `is_sign_positive`, `is_sign_negative`, `classify`
- **Sign and absolute value:** `abs`, `signum`, `copysign`
- **Rounding:** `floor`, `ceil`, `round`, `trunc`, `fract`
- **Min / max:** `min`, `max`, `clamp`
- **Power and root:** `sqrt`, `cbrt`, `powi`, `powf`, `hypot`, `recip`
- **Exponential and logarithm:** `exp`, `exp2`, `exp_m1`, `ln`, `ln_1p`, `log`, `log2`, `log10`
- **Trigonometry:** `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sin_cos`
- **Hyperbolic:** `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
- **Fused multiply-add:** `mul_add` (lowers to `llvm.fma` when the target supports it)
- **Angle conversion:** `to_degrees`, `to_radians`

**Constants** are associated constants on `f32` and `f64`, not free items:

```sploosh
let circumference = 2.0f64 * f64::PI * radius;
let eps = f64::EPSILON;
```

Available constants include `PI`, `TAU`, `E`, `SQRT_2`, `FRAC_1_SQRT_2`, `LN_2`, `LN_10`,
`LOG2_E`, `LOG10_E`, `INFINITY`, `NEG_INFINITY`, `NAN`, `MIN`, `MIN_POSITIVE`, `MAX`,
`EPSILON`, `MANTISSA_DIGITS`, `DIGITS`, `RADIX`. See `stdlib/math.md` for the full list.

**Example:**

```sploosh
fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx*dx + dy*dy).sqrt()
}

fn polar_to_cartesian(r: f64, theta: f64) -> (f64, f64) {
    let (s, c) = theta.sin_cos();  // fuses to llvm.sincos
    (r * c, r * s)
}
```

**On-chain restriction.** Floating-point math methods are a **compile error inside
`onchain` modules**. Transcendentals are not bit-reproducible across LLVM versions,
platforms, and fast-math settings, and on-chain determinism is non-negotiable — any drift
would break consensus. Inside `onchain`, use the integer math methods below. The
`@fast_math` attribute is similarly forbidden in `onchain` (see §12.1, §12.3).

**Integer math methods.** In addition to the overflow-related methods in §4.8, all
integer types (`i8`..`i128`, `u8`..`u128`, `u256`) support the following math and
bit-manipulation methods, all of which are available **on every target including `onchain`**:

- **Arithmetic:** `abs` (signed types only, returns same type — `i32::MIN.abs()` aborts under checked arithmetic), `min`, `max`, `clamp`, `pow` (checked exponentiation)
- **Roots and logs:** `isqrt` (integer square root, floor), `ilog2`, `ilog10` (integer logarithms, abort on zero)
- **Bit counting:** `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`
- **Bit rotation and byte order:** `rotate_left`, `rotate_right`, `swap_bytes`, `to_be`, `to_le`, `from_be`, `from_le`

```sploosh
let half = amount.clamp(0u256, u256::MAX / 2u256);
let shift = capacity.ilog2();           // usable on-chain
let word = hash.rotate_left(13);        // u256 bit tricks for crypto
```

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

**Actors are off-chain primitives.** The `actor` keyword, the `spawn`, `send`,
`send_timeout`, `select`, and `timeout(ms)` intrinsics, and the `Handle<T>`,
`Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>` types are all **compile
errors inside `onchain` modules**. The `@supervisor` and `@mailbox` attributes are
also rejected on items in `onchain` scope. On-chain execution is synchronous,
single-threaded, and transactional — there is no runtime scheduler. Transitive
imports of native modules that internally use actors are still allowed, provided the
functions called across the `onchain` boundary do not themselves touch actor
intrinsics. See §11.1 and §12.3 for the cross-target restriction surface, and §13.0
for the per-intrinsic context column.

### 8.1a Actor Lifecycle States

Every actor observable through a `Handle<T>` is always in one of three states:

| State | Meaning |
|---|---|
| `INITIALIZING` | `spawn` has returned a handle, but `init` has not yet produced the initial `Self`. Incoming messages queue in the mailbox. |
| `READY` | `init` has returned. The actor is processing messages from its mailbox under the normal one-handler-at-a-time rule. |
| `DEAD` | The actor has terminated and will never process another message. Its state has been dropped. |

**`init` is infallible in signature** — it returns `Self`, not `Result<Self, E>`,
and it is not `async`. Writing `async fn init(...)` on an actor is a compile error.
`init` may perform synchronous work that can *fail at runtime* (bounds checks,
overflow checks, `assert` failures); any such failure transitions the actor
directly from `INITIALIZING` to `DEAD` without ever reaching `READY`. Recoverable
initialization should be modeled by storing an `Option<T>` field and running a
handshake message after spawn (see §8.7a for how supervisors observe init
failures).

**Mailbox queuing during `INITIALIZING`.** Because `spawn` returns the handle
immediately (§8.11) while `init` runs asynchronously on the scheduler, messages
sent to a still-initializing actor are simply placed in its mailbox and delivered
once the actor enters `READY`. There is a happens-before edge from the completion
of `init` to the first handler dispatch: the first message handler cannot observe
partially-constructed state. Request/reply calls from other actors block until the
target reaches `READY` (or `DEAD`).

**Handles returned from `spawn` may be observationally dead.** If `init` panics
before returning `Self`, the actor transitions `INITIALIZING → DEAD` and the first
call on the handle observes the dead state — request/reply returns
`Err(ActorError::Dead)`, `send` silently drops (§8.2, §8.11). The spawner does not
receive a synchronous error from `spawn` itself; the handle is the only observation
surface. A supervised actor that dies in `init` is reported to its supervisor as if
a `READY` child had died, and init-failures count toward the supervisor's
`max_restarts` window (§8.7, §8.7a).

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
- **`send` is only valid on `&mut self` methods.** Applying `send` to an `&self`
  method (e.g. `send handle.get()`) is a compile error: an `&self` call has a
  meaningful return value and no mutation, so discarding its reply is never the
  author's intent.
- Sending to a dead actor: request/reply returns `Err(ActorError::Dead)`. `send` silently drops.

**Handle drop semantics.** Dropping a `Handle<T>` — including the last live
handle in the program — has **no effect on the actor's lifetime**. Actors
terminate only via (a) runtime failure (bounds, overflow, failed `assert`),
(b) supervisor termination under the applicable strategy (§8.7), or (c) runtime
shutdown when `main()` returns (§8.11). An actor that is reachable from no live
handle and has an empty mailbox is said to be **orphaned**; it continues running
until the runtime shuts down. Orphaned actors are a known tradeoff of the
non-reference-counted handle model — clean shutdown is the supervisor's job. A
future amendment may introduce explicit self-termination; v0.4.3 deliberately
does not.

**Message ownership rules.** The rule is keyed to the method's receiver:

- **`&mut self` methods** — which may be invoked via `send` (fire-and-forget),
  `send_timeout`, or a direct request/reply call — **must use owned types** for
  every parameter. `&T`, `&mut T`, and any type containing a non-`'static`
  reference are compile errors. The reason is that `send` is the only Sploosh
  construct where a call can outlive the caller's stack frame; forbidding
  references on `&mut self` methods blocks the dangling-reference class at the
  receiver boundary.
- **`&self` methods** — which are synchronous request/reply only (see the handle
  rules above) — **may take reference parameters**. The caller blocks until the
  reply arrives, so the caller's stack frame is guaranteed to outlive the call,
  and references remain sound even when the actor internally `.await`s the
  reply (§8.10). Standard borrow-checker rules apply on the caller side.
- **Private (non-`pub`) methods** are called only during message handling on the
  actor's own scope and may use references freely. This rule is unchanged.

Adding `send` capability to a method with a reference parameter is a compile
error, and adding a reference parameter to a method that is already
`send`-callable is also a compile error. There is no hidden escape hatch.

```sploosh
actor Logger {
    entries: Vec<String>,
    fn init() -> Self { Logger { entries: Vec::new() } }

    // CORRECT: owned String
    pub fn log(&mut self, msg: String) { self.entries.push(msg); }

    // COMPILE ERROR: &str is a reference — not allowed in &mut self
    // pub fn log_ref(&mut self, msg: &str) { ... }

    // OK: &self request/reply methods may take references.
    pub fn count_matching(&self, needle: &str) -> u64 {
        self.entries.iter()
            |> filter(|e| e.contains(needle))
            |> count
    }

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

Actors can be generic. **Every type parameter on an `actor` declaration must be
`Send`** (not only the ones that appear in `pub` method signatures), because the
actor's state fields may hold values of those types and those fields move across
scheduler threads when the actor is migrated between cores. `K`, `V`, and any
other generic parameter must therefore carry a `Send` bound. This is a
conservative rule but zero-cost at the usage site — the same bound is already
required in practice.

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

    // OK under §8.2: &self methods may take reference parameters.
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

### 8.5 Channels

`Channel<T>` is a typed, bounded, multi-producer single-consumer (MPSC) queue.
Distinct from actor mailboxes — channels are for explicit data flow between tasks.

```sploosh
let (tx, rx): (Sender<String>, Receiver<String>) = Channel::new(100);  // capacity 100

// Sending (blocks if full, returns Err if receiver dropped)
tx.send("hello".into())?;

// Receiving (blocks until message available, returns Err if all senders dropped)
let msg = rx.recv()?;
```

**Channel rules:**
- `Channel::new(capacity)` returns `(Sender<T>, Receiver<T>)`.
- `Sender<T>` implements `Clone` and `Send`. Multiple producers can hold clones.
- `Receiver<T>` does NOT implement `Clone`. Single consumer only.
- When the channel is full, `send` blocks the sender until space is available (backpressure).
- `send_timeout(tx.send(val), duration_ms)` returns `Result<(), SendError>` where
  `SendError` has variants `Timeout` (bounded wait elapsed) and `Dead`
  (destination actor died — raised for actor-targeted `send_timeout`, see §8.11).

### 8.6 Select (multiplexed receive)

`select` waits on multiple channel receivers and timeouts simultaneously:

```sploosh
select {
    msg = rx1.recv() => handle_a(msg),
    msg = rx2.recv() => handle_b(msg),
    _ = timeout(5000) => return Err(AppError::Timeout),
}
```

**Select rules:**
- Arms are checked **top-to-bottom deterministically** (not round-robin). When
  multiple arms are simultaneously ready, the first textually listed ready arm
  wins every time. This makes `select` reproducible under test and avoids
  randomized scheduling hazards.
- If no arms are ready, `select` blocks until one becomes available or a timeout fires.
- `timeout(ms)` is a compiler intrinsic usable only inside `select` arms.

### 8.7 Supervision

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60)
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

**Supervision strategies:**

| Strategy | Behavior |
|---|---|
| `one_for_one` | Restart only the failed child |
| `one_for_all` | Restart all children when one fails |
| `rest_for_one` | Restart the failed child and all children started after it |

**Parameters:**
- `max_restarts`: maximum restarts within `window_secs` before the supervisor itself dies (default: 5).
- `window_secs`: time window for counting restarts (default: 60). The window is
  **sliding**, not fixed — each restart is tagged with its wall-clock timestamp,
  and the supervisor counts restarts whose timestamps fall within the last
  `window_secs` seconds of the current time. There are no window-reset
  boundaries to abuse with timing.
- When a supervisor dies, it propagates to ITS supervisor (cascading failure).
- If the top-level supervisor dies, the runtime returns an error from `main()`.

### 8.7a Restart Semantics

When a supervisor restarts a child under any strategy (`one_for_one`,
`one_for_all`, or `rest_for_one`), the runtime performs these steps in order:

1. **Drop the failed actor's state.** RAII runs via any `Drop` impls on state
   fields. The failed actor's mailbox is discarded (consistent with §8.8 —
   pending messages are lost).
2. **Run a fresh `init`** with the arguments the supervisor originally used to
   spawn the child. The new instance begins in `INITIALIZING` per §8.1a and
   transitions to `READY` once `init` completes. The new state is **fresh** —
   there is no preservation of fields across restart. This matches OTP's default
   semantics and avoids the bug class where a corrupted state field is
   "restarted" into a state that caused the crash.
3. **Replace the supervisor's stored handle.** Any `Handle<T>` that was cloned
   out of the supervisor *before* the crash is **permanently dead** — calls on
   those old handles return `Err(ActorError::Dead)` (§8.8) or silently drop for
   `send` (§8.2). Callers that need to reach the restarted actor must re-fetch
   the handle from the supervisor's public API. Blocked senders waiting on the
   dead actor's mailbox are **not transparently redirected** to the new
   instance (§8.11).

The mechanism the supervisor uses to remember each child's construction
arguments (closures, tuples, factory types) is deliberately left to the runtime.
The spec commits only to the observable contract: *same arguments, fresh state,
new handle*.

**Init failures count toward `max_restarts`.** A child that dies in `init`
(§8.1a) is reported to its supervisor exactly as if a `READY` child had died.
This prevents infinite restart storms when `init` consistently panics on bad
configuration: after `max_restarts` failures in `window_secs`, the supervisor
itself dies and the failure cascades to its own supervisor.

**`rest_for_one` ordering requirement.** For `rest_for_one` to be well-defined,
the supervisor must spawn its children in a deterministic order and track them
in an ordered collection (typically `Vec<Handle<T>>`, as in the §8.7 example).
Supervisors that spawn children dynamically into unordered structures (e.g.
`Map<K, Handle<T>>`) have no observable "started after" ordering;
`rest_for_one` on such a supervisor emits a **compile-time warning** and falls
back to `one_for_one` semantics at runtime. The intent is that `rest_for_one`
should always describe a meaningful relationship, not an accidental one.

### 8.8 Actor Failure and Recovery

Actors can fail due to runtime errors (out-of-bounds access, integer overflow,
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
    SelfCall,                       // Direct re-entrant self-call detected (§8.10.1)
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

### 8.9 Async/Await (for non-actor async)

```sploosh
async fn fetch_data(url: &str) -> Result<Response, NetError> {
    let conn = net::connect(url).await?;
    let response = conn.get("/api/data").await?;
    Ok(response)
}
```

**Async task spawning** (non-actor):

```sploosh
let handle: JoinHandle<String> = spawn async {
    let data = fetch_data("https://api.example.com").await?;
    Ok(data.body)
};

let result = handle.await?;   // wait for task completion
```

### 8.10 Async-Actor Integration

`.await` is allowed inside actor message handlers. While awaiting, the actor does
NOT process other messages — it remains "busy" on the current message. This preserves
the single-threaded-per-actor guarantee.

```sploosh
actor DataFetcher {
    cache: Map<String, String>,
    fn init() -> Self { DataFetcher { cache: Map::new() } }

    pub async fn fetch(&mut self, url: String) -> Result<String, AppError> {
        if let Some(cached) = self.cache.get(&url) {
            return Ok(cached.clone());
        }
        let data = net::get(&url).await?;    // actor is busy during await
        self.cache.insert(url, data.clone());
        Ok(data)
    }
}
```

**Rules:**
- While an actor is awaiting, its mailbox is not drained. Messages queue up.
- If an actor needs concurrent I/O and message processing, spawn a separate async task
  and `send` results back.
- Async functions cannot hold `&mut` borrows across `.await` points (borrow checker enforced).

### 8.10.1 Re-entrant Calls and Deadlock

Because an actor holds its mailbox "busy" across a handler's entire execution
(including every `.await` point, §8.10), any synchronous request/reply call
from a handler back into the same actor deadlocks: the caller is waiting for
itself to return before it will process the next message. Sploosh detects the
direct case at runtime and makes the indirect cases the author's responsibility.

**Direct self-calls** (actor A's handler makes a request/reply call on its own
`Handle<A>`, or on any cloned copy of it) return **`Err(ActorError::SelfCall)`
immediately, without blocking**. The runtime compares the target handle's actor
identity against the currently-executing actor on the scheduler thread; the
check is O(1) and free in the fast path. This catches the common accident of
using `self.handle.method(args)` where `self.method(args)` was intended.

```sploosh
actor Recorder {
    entries: Vec<String>,
    self_handle: Option<Handle<Recorder>>,

    fn init() -> Self { Recorder { entries: Vec::new(), self_handle: None } }

    pub async fn append(&mut self, msg: String) -> Result<u64, ActorError> {
        self.entries.push(msg);
        // WRONG: direct self-call via a cloned handle deadlocks on itself.
        // The runtime returns Err(ActorError::SelfCall) instead of hanging.
        // let n = self.self_handle.as_ref().unwrap().count()?;

        // CORRECT: call the local method directly on self.
        Ok(self.entries.len() as u64)
    }

    pub fn count(&self) -> u64 { self.entries.len() as u64 }
}
```

**Multi-actor cycles** (A awaits B, B awaits A; or longer chains) are **not
detected** by the v0.4.3 runtime. Such cycles block indefinitely until an outer
`send_timeout` or user-level timeout fires. The language will not silently
recover: authors must structure actor communication as a DAG, or break the
cycle with fire-and-forget `send` so that no chain of synchronous waits can
close. Cycle detection is expensive (wait-for graph maintenance, false
positives under temporary pauses) and is deliberately deferred until
operational experience justifies the cost.

**Fire-and-forget self-sends are legal and do not deadlock.** A handler may
enqueue a message to itself via `send self.handle.method(args)` — the message
is placed in the actor's own mailbox and processed on the next handler turn
after the current one returns. This is the correct pattern for self-scheduling
work, splitting long computations, or retrying a handler with modified
arguments.

This rule is distinct from the on-chain reentrancy guard in §11.3; on-chain
execution has no actors and no scheduler, so the two mechanisms never overlap.

### 8.11 Runtime Architecture

The actor runtime is the execution engine for all actors and async tasks.

**Scheduler:** M:N work-stealing model.
- One scheduler thread per available CPU core (configurable: `[runtime] threads = N` in `sploosh.toml`).
- Each thread has a bounded, lock-free local run queue (default: 256 tasks).
- Idle schedulers steal tasks from busy queues in FIFO order (oldest tasks first).
- Actors are green threads. An actor processes one message handler to completion, then yields.
- Non-actor async tasks share the same scheduler pool.
- WASM target uses a single-threaded cooperative scheduler (no OS threads in browser).

**Observable guarantees:**
- Messages from the same sender to the same actor are processed in send order (per-sender FIFO).
- Messages from different senders have no ordering guarantee.
- `spawn` returns a `Handle<T>` immediately. The actor's `init` function runs
  asynchronously on the scheduler — it does not block the spawner.

**Mailboxes:**
- Each actor has a bounded, lock-free MPSC mailbox. Default capacity: 1024 messages.
- Configurable per actor with `@mailbox(capacity: N)`.
- When full: `send` (fire-and-forget) blocks the sender until space is available (backpressure).
- `send_timeout(handle.method(args), duration_ms)` returns `Result<(), SendError>`
  with variants `Timeout` and `Dead` (§8.5).
- Sending to a dead actor: `send` drops immediately (no block). Request/reply returns
  `Err(ActorError::Dead)` immediately.

**Death while sender blocked.** If the destination actor dies while a sender is
blocked on its full mailbox, the sender **wakes immediately** regardless of the
mailbox's current fill state. Wake semantics by call style:

- `send handle.method(args)` (fire-and-forget, blocking on backpressure): the
  message is silently dropped, the sender's `send` call returns `()`, and
  execution continues.
- `send_timeout(handle.method(args), ms)`: returns `Err(SendError::Dead)`
  immediately, without running to the full timeout.
- Synchronous request/reply on an `&self` or `&mut self` method: returns
  `Err(ActorError::Dead)`.

Wake order across multiple blocked senders is **unspecified**; blocked senders
are not woken FIFO or in any observable order, and fairness is not guaranteed.
**Supervisor restart does not redirect blocked senders.** If the runtime
restarts the actor while a sender is blocked on its (now-dead) mailbox, the
blocked sender still wakes with `Err(...::Dead)` — the message is never
transparently re-delivered to the new instance. To reach the restarted actor,
the caller must re-fetch the new handle from the supervisor (§8.7a).

**Memory model:**
- No garbage collector. Deterministic drop via ownership.
- Default allocator: system allocator on native, linear memory on WASM, bump allocator on-chain.
- Actor messages are moved (zero-copy). The sender gives up ownership; the receiver takes it.

**Runtime lifecycle:**
- The runtime starts when `main()` begins and shuts down when `main()` returns.
- `Ok(())`: graceful shutdown — supervisors notified, actors finish current message
  (configurable timeout, default 30 seconds).
- `Err(e)`: immediate shutdown — all actors killed, pending messages dropped.
- There is no explicit `Runtime::new()`. The `main` function is the entry point.

### 8.11a Blocking Operations in Handlers

Actor message handlers run on scheduler threads that also execute other actors.
A handler that blocks its OS thread starves every other actor on that core.
Sploosh forbids blocking operations in handlers by construction rather than by
attribute marking.

**Standard library.** The standard library exposes **no synchronous blocking
I/O surface**. `std::fs`, `std::net`, `std::io`, `std::db`, and `std::web` are
async-only (§13.2): their methods return futures and require `.await`. There
is nothing to forbid at the type level — the absence of a sync API *is* the
forbid.

**FFI.** `extern "C"` functions are synchronous by default (§4.9). Calling a
synchronous `extern "C"` function from inside an actor message handler — either
directly, or transitively through any function the handler calls — is a
**compile error**. FFI that needs to be safe to call from handlers must be
declared `extern "C" async`; the compiler then emits an awaitable wrapper that
offloads the underlying call to the runtime's blocking pool, so the scheduler
thread is never pinned.

```sploosh
extern "C" {
    fn native_decompress(buf: &[u8]) -> Result<Vec<u8>, FfiError>;  // sync
}

extern "C" async {
    fn native_fetch_blocking(url: &str) -> Result<Vec<u8>, FfiError>;  // handler-safe
}

actor Loader {
    fn init() -> Self { Loader {} }

    pub async fn load(&mut self, url: String) -> Result<Vec<u8>, FfiError> {
        // COMPILE ERROR: native_decompress is sync and would pin the scheduler thread.
        // let data = native_decompress(&buf)?;

        // OK: native_fetch_blocking is async-wrapped.
        let bytes = native_fetch_blocking(&url).await?;
        Ok(bytes)
    }
}
```

**Spin loops and busy waits** are legal but discouraged; they violate no rule,
but they starve other actors on the same scheduler thread. Use `.await`, or a
`timeout(ms)` arm in a `select`, to yield instead.

**Scheduler yielding.** An actor that makes no `.await` call during a handler
runs to handler completion and then yields (see the "An actor processes one
message handler to completion, then yields" rule above). Handlers that cannot
reach an `.await` point in bounded time should be split into smaller messages
or moved to a separate `spawn async { }` task and communicate results back via
`send`.

A future revision may introduce an explicit `spawn_blocking async { }`
intrinsic for offloading ad-hoc blocking work; in v0.4.3, the only way to
invoke blocking code from a handler is via an `extern "C" async` wrapper or by
forwarding to a non-actor `spawn async` task.

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

### 10.4 File Resolution

- `mod foo;` (with semicolon) — look for `foo.sp` adjacent to the current file,
  then `foo/mod.sp`. First match wins.
- `mod foo { ... }` (with body) — inline definition. No file lookup.
- `crate::` — refers to the crate root (`src/main.sp` or `src/lib.sp`).
- `self::` — refers to the current module.
- `super::` — refers to the parent module.
- `pub use crate::models::User;` — re-export for cleaner public APIs.

### 10.5 Trait Coherence (Orphan Rules)

You can only implement a trait for a type if you defined **the trait** or **the type**
(or both) in your crate.

- You CAN implement your trait for a foreign type.
- You CAN implement a foreign trait for your type.
- You CANNOT implement a foreign trait for a foreign type.
- Blanket impls (`impl<T: Foo> Bar for T`) are allowed only if you own `Bar`.

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

**Concurrency primitives are not available on-chain.** On-chain execution is
synchronous, single-threaded, and transactional — there is no runtime
scheduler. The following are all compile errors inside `onchain` modules:

- The `actor` keyword and any `actor Foo { ... }` declaration.
- The `spawn`, `send`, `send_timeout`, `select`, and `timeout(ms)` intrinsics.
- The `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, and `JoinHandle<T>`
  types in storage fields, event fields, function signatures, or local bindings.
- The `@supervisor` and `@mailbox` attributes on any item in `onchain` scope.
- `extern "C"` blocks of any kind, including `extern "C" async` (§4.9).
- `.await` outside of the `onchain` module — on-chain functions must be
  synchronous end-to-end within a transaction.

Transitive imports of native modules that internally use actors are still
allowed, provided the functions called across the `onchain` boundary do not
themselves touch any of the constructs above. The forbid is on *spawning
inside `onchain`*, not on depending on actor-using code through pure-function
boundaries. See §8.1 for the actor-side statement of this rule, §12.3 for the
stdlib restriction surface, and §13.0 for the per-intrinsic context column.

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
| `@mailbox(capacity: N)` | Set actor mailbox capacity (default: 1024) |
| `@overflow(wrapping)` | Opt function into wrapping arithmetic (compile error on-chain) |
| `@fast_math(flags)` | Enable LLVM fast-math flags for floating-point operations (compile error on-chain) |

**Fast-math flags.** The `@fast_math` attribute enables LLVM fast-math flags on the floating-point operations
inside a function body, trading IEEE 754 strictness for speed. The attribute takes zero
or more flag names corresponding 1:1 to LLVM's fast-math flags:

| Flag | Meaning | Safe to enable? |
|---|---|---|
| `contract` | Allow contraction, e.g. `x*y + z` → `fma(x, y, z)` | Yes — fuses precision-preserving ops |
| `afn` | Allow approximate math function implementations (~1 ULP error on transcendentals) | Usually |
| `reassoc` | Allow reordering of associative operations | Only if you know the inputs |
| `arcp` | Allow `x/y` → `x * (1/y)` | Only if reciprocal error is acceptable |
| `nnan` | Assume no operand is NaN (UB if violated) | Requires domain reasoning |
| `ninf` | Assume no operand is infinity (UB if violated) | Requires domain reasoning |
| `nsz` | Ignore sign of zero (treats −0.0 and +0.0 as equivalent) | Requires domain reasoning |

**Bare `@fast_math` is shorthand for `@fast_math(contract, afn)`** — the safe subset
that enables FMA fusion and approximate transcendentals without introducing undefined
behavior on NaN or infinity inputs. These are the two flags that deliver the bulk of
the performance win on typical numeric code. All other flags must be opted in explicitly
because they require the author to reason about the numerical domain.

```sploosh
@fast_math                          // equivalent to @fast_math(contract, afn)
fn length_sq(v: &[f64]) -> f64 {
    let mut sum = 0.0f64;
    for x in v { sum = sum + x * x; }   // compiler may fuse to FMA
    sum
}

@fast_math(contract, afn, reassoc)   // allow reduction reordering too
fn dot(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0f64;
    for i in 0..a.len() { acc = acc + a[i] * b[i]; }
    acc
}
```

**Scope:** `@fast_math` applies to floating-point operations inside the annotated
function body only. It is **not inherited** by called functions — a `@fast_math` caller
does not relax the semantics of a strict-math callee.

**On-chain:** `@fast_math` is a **compile error inside `onchain` modules**. On-chain
float determinism requires strict IEEE 754 semantics on every target; fast-math flags
would allow bit-level drift across LLVM versions and break consensus. See §12.3.

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

Available inside `onchain`: `std::math` (integer math only — `abs`, `min`, `max`, `clamp`,
`pow`, `isqrt`, `ilog2`, `count_ones`, and the other methods listed in §4.10),
`std::crypto`, `std::chain`, `std::collections`, and all core types.

**Concurrency primitives forbidden in `onchain`.** In addition to the I/O-bound
stdlib modules above, the entire actor and async runtime surface is a compile
error inside `onchain` modules: the `actor` keyword, the `spawn`, `send`,
`send_timeout`, `select`, `timeout(ms)` intrinsics, the `Handle<T>`,
`Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>` types, and the
`@supervisor`, `@mailbox` attributes. `extern "C"` and `extern "C" async` FFI
blocks are also forbidden on-chain (§4.9, §11.1). On-chain execution is
synchronous, single-threaded, and transactional — there is no scheduler for any
of these constructs to run on. See §8.1 and §11.1 for the cross-references.

**Forbidden inside `onchain`:** every floating-point math method listed in §4.10 is a
compile error inside `onchain` modules — classification (`is_nan`, `is_finite`, ...),
sign and absolute value (`abs`, `signum`, `copysign`), rounding (`floor`, `ceil`, `round`,
`trunc`, `fract`), min/max/clamp, power/root (`sqrt`, `cbrt`, `powi`, `powf`, `hypot`,
`recip`), exp/log, trig, hyperbolic, `mul_add`, and angle conversion. The rule is
intentionally uniform: even the IEEE 754-deterministic methods (e.g., `sqrt`, `min`,
`max`, `abs`) are banned on-chain so that implementers and auditors never have to
reason about which subset is safe. Transcendentals are not bit-reproducible across
LLVM versions, platforms, and fast-math settings, and any drift would break on-chain
consensus. The `@fast_math` attribute is similarly forbidden in `onchain` for the same
reason (§12.1). `f32`/`f64` values themselves may still be stored in fields, compared
with `==`/`<`/`>`, and passed as arguments inside `onchain` code — only the §4.10
method calls are rejected. Use the integer math methods from §4.10 for all on-chain
numeric work.

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

### 13.0 Compiler Intrinsics

Compiler intrinsics are built-in functions and constructs that look like regular code but
are implemented by the compiler. They are not part of the standard library and cannot be
user-defined.

**General intrinsics:**

| Intrinsic | Signature | Context | Purpose |
|---|---|---|---|
| `format(template, ...)` | Variadic, compile-time checked | All | String formatting |
| `print(value)` | `fn(impl Display)` | native, wasm | Write to stdout + newline |
| `assert(cond)` | `fn(bool)` | native, wasm | Abort/actor death on false |
| `assert(cond, msg)` | `fn(bool, &str)` | native, wasm | Assert with message |
| `vec![a, b, c]` | `-> Vec<T>` | All | Vec literal |
| `vec![val; count]` | `(T: Clone, usize) -> Vec<T>` | All | Vec repeat |

**Concurrency intrinsics:**

| Intrinsic | Signature | Context | Purpose |
|---|---|---|---|
| `spawn expr` | `ActorInit -> Handle<T>` | not onchain | Spawn actor |
| `spawn async { block }` | `-> JoinHandle<T>` | not onchain | Spawn async task |
| `send expr` | `ActorMethod -> ()` | not onchain | Fire-and-forget message |
| `send_timeout(expr, ms)` | `-> Result<(), SendError>` | not onchain | Bounded send |
| `select { arms }` | Special syntax | not onchain | Multiplexed receive |
| `timeout(ms)` | `fn(u64) -> TimeoutFuture` | not onchain | Timeout in select |

**On-chain intrinsics:**

| Intrinsic | Return Type | Context | Purpose |
|---|---|---|---|
| `emit Event { fields }` | `()` | onchain | Emit blockchain event |
| `ctx::caller()` | `Address` | onchain | Transaction caller |
| `ctx::self_address()` | `Address` | onchain | Contract address |
| `ctx::timestamp()` | `u256` | onchain | Block timestamp |
| `ctx::block_number()` | `u256` | onchain | Block number |
| `ctx::value()` | `u256` | onchain, EVM, @payable | ETH sent (wei) |
| `ctx::gas_remaining()` | `u256` | onchain, EVM | Remaining gas |
| `ctx::chain_id()` | `u256` | onchain, EVM | Chain ID |
| `ctx::lamports()` | `u64` | onchain, SVM | SOL sent |
| `ctx::program_id()` | `Address` | onchain, SVM | Program address |
| `ctx::signer()` | `Address` | onchain, SVM | Transaction signer |
| `storage::get(field, key)` | Varies | onchain | Read persistent state |
| `storage::set(field, key, val)` | `()` | onchain | Write persistent state |
| `chain::call(addr, fn, args)` | `Result<T, E>` | onchain | Cross-contract call |

**Math intrinsics:**

All floating-point math methods in §4.10 are compiler intrinsics that lower directly to
LLVM intrinsics. The `f32` forms lower to `llvm.*.f32` and the `f64` forms lower to
`llvm.*.f64`; only the `f64` forms are listed below for brevity.

| Intrinsic | Signature | Lowers to | Context |
|---|---|---|---|
| `f64.sqrt()` | `fn(f64) -> f64` | `llvm.sqrt.f64` (correctly rounded) | not onchain |
| `f64.abs()` | `fn(f64) -> f64` | `llvm.fabs.f64` | not onchain |
| `f64.mul_add(b, c)` | `fn(f64, f64, f64) -> f64` | `llvm.fma.f64` (correctly rounded) | not onchain |
| `f64.sin()` | `fn(f64) -> f64` | `llvm.sin.f64` | not onchain |
| `f64.cos()` | `fn(f64) -> f64` | `llvm.cos.f64` | not onchain |
| `f64.tan()` | `fn(f64) -> f64` | `llvm.tan.f64` | not onchain |
| `f64.sin_cos()` | `fn(f64) -> (f64, f64)` | `llvm.sincos.f64` | not onchain |
| `f64.exp()` | `fn(f64) -> f64` | `llvm.exp.f64` | not onchain |
| `f64.exp2()` | `fn(f64) -> f64` | `llvm.exp2.f64` | not onchain |
| `f64.ln()` | `fn(f64) -> f64` | `llvm.log.f64` | not onchain |
| `f64.log2()` | `fn(f64) -> f64` | `llvm.log2.f64` | not onchain |
| `f64.log10()` | `fn(f64) -> f64` | `llvm.log10.f64` | not onchain |
| `f64.powf(e)` | `fn(f64, f64) -> f64` | `llvm.pow.f64` | not onchain |
| `f64.powi(e)` | `fn(f64, i32) -> f64` | `llvm.powi.f64` | not onchain |
| `f64.floor()` | `fn(f64) -> f64` | `llvm.floor.f64` | not onchain |
| `f64.ceil()` | `fn(f64) -> f64` | `llvm.ceil.f64` | not onchain |
| `f64.trunc()` | `fn(f64) -> f64` | `llvm.trunc.f64` | not onchain |
| `f64.round()` | `fn(f64) -> f64` | `llvm.round.f64` | not onchain |
| `f64.copysign(s)` | `fn(f64, f64) -> f64` | `llvm.copysign.f64` | not onchain |
| `f64.min(b)` | `fn(f64, f64) -> f64` | `llvm.minnum.f64` | not onchain |
| `f64.max(b)` | `fn(f64, f64) -> f64` | `llvm.maxnum.f64` | not onchain |

Remaining methods from §4.10 (`asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`,
`asinh`, `acosh`, `atanh`, `cbrt`, `hypot`, `exp_m1`, `ln_1p`, `log`, `recip`, `fract`,
`signum`, `is_nan`, `is_finite`, `is_infinite`, `is_normal`, `is_sign_positive`,
`is_sign_negative`, `classify`, `to_degrees`, `to_radians`, `clamp`) are also compiler
intrinsics. Where a direct LLVM intrinsic exists (e.g., `llvm.asin`, `llvm.atan2`,
`llvm.sinh`), the method lowers to it; otherwise the compiler lowers to LLVM libc libm.

**Integer math intrinsics** (all available on-chain):

| Intrinsic | Signature | Lowers to | Context |
|---|---|---|---|
| `uN.count_ones()` | `fn(uN) -> u32` | `llvm.ctpop.iN` | All |
| `uN.leading_zeros()` | `fn(uN) -> u32` | `llvm.ctlz.iN` | All |
| `uN.trailing_zeros()` | `fn(uN) -> u32` | `llvm.cttz.iN` | All |
| `uN.rotate_left(n)` | `fn(uN, u32) -> uN` | `llvm.fshl.iN` | All |
| `uN.rotate_right(n)` | `fn(uN, u32) -> uN` | `llvm.fshr.iN` | All |
| `uN.swap_bytes()` | `fn(uN) -> uN` | `llvm.bswap.iN` | All |
| `uN.isqrt()` | `fn(uN) -> uN` | Compiler-provided | All |
| `uN.ilog2()` | `fn(uN) -> u32` | Derived from `ctlz` | All |

Remaining integer methods from §4.10 (`abs`, `min`, `max`, `clamp`, `pow`, `ilog10`,
`count_zeros`, `to_be`, `to_le`, `from_be`, `from_le`) are also compiler intrinsics on
every integer type, available on all targets including `onchain`. They are
compiler-provided without a 1:1 LLVM intrinsic mapping — the compiler emits the
equivalent sequence of primitive operations and lets the optimizer handle the rest.

**Notes:**
- `vec![]` uses `![]` syntax. This is the only intrinsic with this form. No other "macros" exist.
- `format()` template strings are validated at compile time — mismatched `{}` count is an error.
- `print()` and `assert()` are not available in `onchain` modules (compile error).
- `assert()` failure in an actor causes actor death. In non-actor code, it aborts the program.
- The optimizer may fuse adjacent `.sin()` and `.cos()` calls on the same input into a single `llvm.sincos` call. Math calls inside loops are auto-vectorized when the target has a SIMD libm (SVML on Intel, libmvec on glibc).
- Constant expressions involving math intrinsics are folded at compile time: `(0.0f64).sin()` becomes `0.0` during codegen, with no runtime call.

### 13.1 Prelude (auto-imported)

```
Option, Some, None
Result, Ok, Err
String, Vec, Map, Set, Box
print, format, assert
Display, Debug, Clone, Copy, Eq, Hash, Ord
From, Into, TryFrom, TryInto
Drop, Error
Fn, FnMut, FnOnce
Iter, FromIter
Send, Sync
Handle, JoinHandle
Channel, Sender, Receiver
Address, u256
FpCategory
```

### 13.2 Core Modules

| Module         | Purpose                                  | Targets |
|----------------|------------------------------------------|---------|
| `std::io`      | File I/O, stdin/stdout                   | native, wasm |
| `std::net`     | TCP/UDP, HTTP client                     | native, wasm |
| `std::json`    | JSON parse/serialize                     | all |
| `std::crypto`  | Hashing, signing, key generation         | all |
| `std::time`    | Timestamps, durations, timers            | native, wasm |
| `std::math`    | Integer math, bit ops, IEEE 754 float methods | integer: all; float: native, wasm |
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

## 16. Grammar (EBNF)

The grammar is split into **syntactic productions** (below) and **lexical productions**
(§16.1). Together they form the complete formal grammar of Sploosh. Every non-terminal
used on the right-hand side of any production has a definition either in this section
or in §16.1.

```ebnf
program        = { item } ;
item           = [ directives ] item_kind ;
item_kind      = fn_def | struct_def | enum_def | trait_def
               | impl_block | mod_def | use_stmt | actor_def
               | onchain_mod | const_def | type_alias | extern_block ;

fn_def         = [ attrs ] [ "pub" ] [ "async" ] "fn" IDENT [ generics ] "(" params ")"
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
trait_item     = fn_sig ( block | ";" ) | "type" IDENT [ ":" bounds ] ";" ;

impl_block     = "impl" [ generics ] [ trait_ref "for" ] type "{" { impl_item } "}" ;
impl_item      = fn_def | "type" IDENT "=" type ";" ;

actor_def      = [ attrs ] "actor" IDENT [ generics ] "{" { actor_item } "}" ;
actor_item     = field_def | fn_def ;

mod_def        = [ "pub" ] "mod" IDENT ( ";" | "{" { item } "}" ) ;
use_stmt       = "use" path [ "::" "{" idents "}" ] ";" ;

onchain_mod    = "onchain" "mod" IDENT "{" { onchain_item } "}" ;
onchain_item   = storage_block | fn_def | event_def ;
storage_block  = "storage" "{" fields "}" ;

extern_block   = "extern" STRING_LIT "{" { extern_fn } "}" ;
extern_fn      = "fn" IDENT "(" params ")" [ "->" type ] ";" ;

type           = prim_type | IDENT [ generics ] | "&" [ lifetime ] [ "mut" ] type
               | "[" type ";" expr "]" | "[" type "]"
               | "(" [ type { "," type } ] ")" | "fn" "(" types ")" "->" type
               | "dyn" IDENT [ generics ] ;
prim_type      = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256"
               | "f32" | "f64" | "bool" | "char" | "str" | "String"
               | "Address" | "()" ;
types          = [ type { "," type } [ "," ] ] ;
type_alias     = [ "pub" ] "type" IDENT [ generics ] "=" type ";" ;
trait_ref      = IDENT [ generics ] ;
generics       = "<" type_params ">" ;
type_params    = type_param { "," type_param } ;
type_param     = IDENT [ ":" bounds ] | lifetime ;
bounds         = bound { "+" bound } ;
bound          = IDENT [ generics ] | lifetime ;

block          = "{" { statement } [ expr ] "}" ;
statement      = let_stmt | expr_stmt | return_stmt | emit_stmt ;
let_stmt       = "let" [ "mut" ] pattern [ ":" type ] "=" expr ";" ;
const_def      = [ "pub" ] "const" IDENT ":" type "=" expr ";" ;
return_stmt    = "return" [ expr ] ";" ;
emit_stmt      = "emit" IDENT "{" field_inits "}" ";" ;
expr_stmt      = expr ";" ;

expr           = literal | IDENT | path_expr
               | expr "." IDENT | expr "(" args ")"  | expr "[" expr "]"
               | expr BINOP expr | UNOP expr | expr "?" | expr "as" type
               | if_expr | if_let_expr | match_expr | block | closure
               | expr "|>" expr
               | "spawn" expr | "spawn" "async" block
               | "send" expr | "recv" expr
               | expr ".await"
               | select_expr
               | "for" pattern "in" expr block
               | "while" expr block | while_let_expr | "loop" block ;

if_expr        = "if" expr block [ "else" ( if_expr | if_let_expr | block ) ] ;
if_let_expr    = "if" "let" pattern "=" expr block [ "else" block ] ;
while_let_expr = "while" "let" pattern "=" expr block ;
match_expr     = "match" expr "{" { match_arm } "}" ;
match_arm      = pattern [ "if" expr ] "=>" ( expr "," | block ) ;
select_expr    = "select" "{" { select_arm } "}" ;
select_arm     = pattern "=" expr "=>" ( expr "," | block ) ;
closure        = [ "move" ] "|" params "|" ( expr | block ) ;

path_expr      = IDENT { "::" IDENT } ;
path           = [ "crate" | "super" | "self" ] { "::" IDENT } ;
args           = [ expr { "," expr } [ "," ] ] ;

BINOP          = "+" | "-" | "*" | "/" | "%"
               | "==" | "!=" | "<" | ">" | "<=" | ">="
               | "&&" | "||"
               | ".." | "..=" ;
UNOP           = "!" | "-" ;

pattern        = "_" | literal | IDENT | [ "ref" ] IDENT
               | IDENT "(" patterns ")" | IDENT "{" field_pats [ ".." ] "}"
               | "(" patterns ")" | pattern "|" pattern ;
patterns       = [ pattern { "," pattern } [ "," ] ] ;
field_pats     = [ field_pat { "," field_pat } [ "," ] ] ;
field_pat      = IDENT [ ":" pattern ] ;
field_inits    = [ field_init { "," field_init } [ "," ] ] ;
field_init     = IDENT [ ":" expr ] ;
idents         = IDENT { "," IDENT } ;

fn_sig         = [ "pub" ] [ "async" ] "fn" IDENT [ generics ] "(" params ")" [ "->" type ] ;
field_def      = [ "pub" ] IDENT ":" type ;
event_def      = [ attrs ] "enum" IDENT "{" variants "}" ;

literal        = INT_LIT [ type_suffix ] | FLOAT_LIT [ type_suffix ]
               | STRING_LIT | CHAR_LIT
               | "true" | "false" | "none" ;
type_suffix    = "i8" | "i16" | "i32" | "i64" | "i128"
               | "u8" | "u16" | "u32" | "u64" | "u128" | "u256"
               | "f32" | "f64" ;

attrs          = { "@" IDENT [ "(" attr_args ")" ] } ;
attr_args      = attr_arg { "," attr_arg } ;
attr_arg       = IDENT [ ":" expr | "=" expr | "(" expr ")" ] | expr ;
directives     = { "#[" IDENT [ "(" dir_args ")" ] "]" } ;
dir_args       = attr_args ;
```

### 16.1 Lexical Productions

The syntactic grammar above uses the terminals `IDENT`, `INT_LIT`, `FLOAT_LIT`,
`STRING_LIT`, `CHAR_LIT`, and `lifetime`. Their lexical grammar is defined here.
Whitespace and comments (see §2.2) may appear between any two tokens and are
discarded by the lexer.

```ebnf
(* Identifiers *)
IDENT          = ASCII_ALPHA_US { ASCII_ALNUM_US } ;
ASCII_ALPHA_US = "A" ... "Z" | "a" ... "z" | "_" ;
ASCII_ALNUM_US = ASCII_ALPHA_US | DIGIT ;

(* Keywords take precedence over IDENT — see §2.3 and §2.7. *)

(* Lifetime annotations *)
lifetime       = "'" IDENT ;

(* Integer literals *)
INT_LIT        = dec_lit | hex_lit | oct_lit | bin_lit ;
dec_lit        = DIGIT { DIGIT | "_" } ;
hex_lit        = "0x" HEX_DIGIT { HEX_DIGIT | "_" } ;
oct_lit        = "0o" OCT_DIGIT { OCT_DIGIT | "_" } ;
bin_lit        = "0b" BIN_DIGIT { BIN_DIGIT | "_" } ;

(* Float literals *)
FLOAT_LIT      = dec_lit "." dec_lit [ exp_part ]
               | dec_lit exp_part ;
exp_part       = ( "e" | "E" ) [ "+" | "-" ] dec_lit ;

(* String and character literals *)
STRING_LIT     = '"' { str_body_char } '"' ;
str_body_char  = UNICODE_SCALAR_EXCEPT_BACKSLASH_QUOTE | escape
               | "\" NEWLINE WHITESPACE ;   (* line continuation *)
CHAR_LIT       = "'" ( UNICODE_SCALAR_EXCEPT_BACKSLASH_APOS | escape ) "'" ;

escape         = "\" ( simple_escape | hex_escape | unicode_escape ) ;
simple_escape  = "n" | "r" | "t" | "\" | '"' | "'" | "0" ;
hex_escape     = "x" HEX_DIGIT HEX_DIGIT ;            (* value must be 0x00..0x7F *)
unicode_escape = "u" "{" HEX_DIGIT { HEX_DIGIT } "}" ; (* 1..6 hex digits, must be a valid Unicode scalar *)

(* Digit classes *)
DIGIT          = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
HEX_DIGIT      = DIGIT | "a" ... "f" | "A" ... "F" ;
OCT_DIGIT      = "0" ... "7" ;
BIN_DIGIT      = "0" | "1" ;
```

**Lexical constraints enforced by the lexer beyond the EBNF above:**

- Underscores in numeric literals must appear between two digits — leading, trailing,
  and consecutive underscores are a compile error.
- `hex_escape` values must be in the range `0x00`–`0x7F` (ASCII only). Use
  `unicode_escape` for values ≥ `0x80`.
- `unicode_escape` values must be a valid Unicode scalar value — surrogate code points
  `0xD800`–`0xDFFF` are rejected, as are values above `0x10FFFF`.
- Literal overflow (the integer value does not fit in its declared or inferred numeric
  type) is a compile error at parse time, not a runtime check.
- `CHAR_LIT` contains exactly one Unicode scalar value. Empty character literals and
  multi-character character literals are compile errors.

See §2.6 for worked examples of each literal form and §2.7 for the identifier rules
in prose.

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
| Minimal lifetime elision (single-source rule) | One ref in + one ref out = same lifetime. Explicit when multiple sources. Covers 95% of cases. |
| Two visibility levels only | `pub` or private. No decision fatigue for the model. |
| Checked arithmetic everywhere | LLMs should generate correct code. `wrapping_*`/`saturating_*` for intentional use. Safety-first. |
| No `unsafe`, safe `extern "C"` | LLMs misuse `unsafe` as escape hatch. Compiler generates safe FFI wrappers. No raw pointers. |
| Bounded mailboxes + backpressure | Unbounded causes OOM. Blocking sender is explicit. `send_timeout` for escape hatch. |
| M:N work-stealing scheduler | BEAM-proven architecture. Lock-free bounded queues. Per-core scheduling. |
| No `Rc<T>`/`Arc<T>` in v0.4 | Actors replace shared ownership. `Handle<T>` is the sharing mechanism. Simpler LLM surface. |
| `as` for numeric casts only | Deeply trained from Rust/C. Scoped to numerics to prevent misuse as type coercion. |
| `vec![]` as compiler intrinsic | No macro system. Single intrinsic simpler than a macro mechanism. Deeply trained from Rust. |
| Channels as bounded MPSC | Go channel mental model. Typed, bounded, backpressure. Distinct from actor mailboxes. |
| Three supervision strategies | Erlang/OTP proven set: `one_for_one`, `one_for_all`, `rest_for_one`. |
| `u256` and `Address` as primitives | Available on all targets. `u256` always checked. `Address` is not an integer. |
| `Box<T>` with `Drop` trait | RAII pattern. Deterministic cleanup. `Drop` + `Copy` mutually exclusive. |
| Standard orphan rule | Match Rust's coherence rule. Deeply trained. Prevents conflicting impls. |

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
| v0.4.0 | **Runtime Specification** (§8.10-8.11): M:N work-stealing scheduler, bounded lock-free mailboxes with backpressure, per-sender FIFO ordering, async-actor integration, runtime lifecycle. **Type System**: `u256`/`Address` primitives (§3.1), `Box<T>` heap allocation (§4.4), `Channel<T>`/`Sender<T>`/`Receiver<T>` (§3.2, §8.5), `Drop` trait (§3.10), associated types in traits (§3.5), standard traits catalog — 30+ traits formally defined (§3.10), `as` numeric casting (§3.11). **Safety**: checked arithmetic everywhere (§4.8), `@overflow(wrapping)` opt-out, `wrapping_*`/`saturating_*`/`checked_*` methods on all integer types. **Ownership**: lifetime elision — single-source rule (§4.5), `Box<T>` with RAII drop semantics, no `Rc<T>`/`Arc<T>`. **FFI**: `extern "C"` with safe wrappers, no `unsafe` keyword, no raw pointers (§4.9). **Concurrency**: typed bounded channels (§8.5), `select` formalized (§8.6), `spawn async {}` for non-actor tasks (§8.9), three supervision strategies (§8.7), `@mailbox(capacity)` attribute, `send_timeout` intrinsic. **Modules**: file resolution rules (§10.4), `pub use` re-exports, trait coherence/orphan rules (§10.5). **Compiler intrinsics catalog** (§13.0): all 25+ intrinsics formally specified with signatures and contexts. **Grammar**: `as` cast, `select_expr`, `spawn async`, `emit_stmt`, `extern_block`, associated `type` in traits/impls, `type_suffix` with `u256`. Keywords 38→40 (`as`, `extern`). |
| v0.4.1 | **std::math module** (§4.10, `stdlib/math.md`): comprehensive IEEE 754 surface on `f32`/`f64` as method-syntax compiler intrinsics that lower directly to LLVM intrinsics (`llvm.sin`, `llvm.sqrt`, `llvm.fma`, `llvm.sincos`, `llvm.pow`, `llvm.log`, etc.), unlocking constant folding, auto-vectorization, and sin+cos fusion. Method categories: classification, sign, rounding, min/max/clamp, power/root (`sqrt`, `cbrt`, `powi`, `powf`, `hypot`, `recip`), exp/log (`exp`, `exp2`, `exp_m1`, `ln`, `ln_1p`, `log2`, `log10`), trig (`sin`, `cos`, `tan`, inverses, `atan2`, `sin_cos`), hyperbolic, `mul_add` (correctly rounded FMA), `to_degrees`/`to_radians`. Constants as associated consts: `f64::PI`, `f64::TAU`, `f64::E`, `f64::EPSILON`, `f64::INFINITY`, `f64::NAN`, etc. **Integer math methods** (§4.10) on all integer types: `abs`, `min`, `max`, `clamp`, `pow` (checked), `isqrt`, `ilog2`, `ilog10`, `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`, `rotate_left`, `rotate_right`, `swap_bytes`. **`@fast_math(flags)` attribute** (§12.1): granular LLVM fast-math flags — `contract`, `afn`, `reassoc`, `arcp`, `nnan`, `ninf`, `nsz`; bare `@fast_math` defaults to the safe subset `contract + afn`; per-function scope, not inherited. **On-chain restriction** (§12.3, §4.10): floating-point math methods and `@fast_math` are compile errors inside `onchain` modules — only integer math is available, for bit-level determinism across LLVM versions and platforms. **§13.0 Compiler Intrinsics**: new Math intrinsics and Integer math intrinsics tables with LLVM lowering targets. No grammar changes — method call and attribute syntax already in §16. Keyword count unchanged (40). |
| v0.4.2 | **Lexical foundation** (new §2.6 Literals, new §2.7 Identifiers): formal prose and grammar for numeric literal formats (decimal, hex `0x`, octal `0o`, binary `0b`, underscore digit separators, exponent notation, type suffixes), string literals with a complete escape sequence table (`\n`, `\r`, `\t`, `\\`, `\"`, `\'`, `\0`, `\xNN` for ASCII, `\u{H...}` for Unicode scalar values, plus `\<newline>` line continuation), character literals (single-quoted Unicode scalar values, surrogates rejected), and identifier grammar (ASCII `[A-Za-z_][A-Za-z0-9_]*`, keyword priority, `_` as wildcard binding, `_name` allowed for intentionally-unused bindings). **Literal overflow is now a compile error** — integer literals that do not fit their declared or inferred type are rejected at parse time. **Float-to-int cast edge cases** (§3.11): NaN → 0, +∞ → target `MAX`, −∞ → target `MIN` (signed) or 0 (unsigned), matching WebAssembly `trunc_sat` semantics. No more undefined behavior on exotic float values. **Address representation** (§3.1): clarified as always 32 bytes big-endian in memory on every target; EVM serialization left-pads the low 20 bytes with 12 zero high bytes (Solidity-compatible); non-zero high bytes are rejected at EVM serialization time; SVM uses the full 32 bytes unchanged. **§16 Grammar completeness**: new §16.1 Lexical Productions block with formal EBNF for `IDENT`, `INT_LIT`, `FLOAT_LIT`, `STRING_LIT`, `CHAR_LIT`, `lifetime`, `escape`, and digit classes. Previously-undefined non-terminals now defined in §16: `path_expr`, `path`, `args`, `types`, `patterns`, `field_pats`, `field_pat`, `field_inits`, `field_init`, `idents`, `fn_sig`, `field_def`, `event_def`, `attr_args`, `attr_arg`, `dir_args`, `prim_type`, `type_alias`, `trait_ref`, `BINOP`, `UNOP`. Case normalization: `EXPR` → `expr` in the array type production. No feature changes — this is a completeness pass unblocking tokenizer and parser implementation. No new keywords (still 40). |
| v0.4.3 | **Actor lifecycle states** (new §8.1a): explicit `INITIALIZING → READY → DEAD` state machine. `fn init(args) -> Self` is **infallible by signature and non-async** — writing `async fn init` is a compile error, and `init` panic transitions directly to `DEAD`. Messages sent to an INITIALIZING actor queue in the mailbox and are delivered once `init` returns; there is a happens-before edge from `init` completion to the first handler dispatch, so handlers never observe partially-constructed state. Handles returned by `spawn` may be immediately dead if `init` panics — first call observes `Err(ActorError::Dead)` or silent drop. **Message ownership rule rewritten by receiver type** (§8.2): `&mut self` methods (which may be invoked via `send`) must use owned parameters; `&self` methods (request/reply only — caller always blocks, stack always outlives the call) **may** take reference parameters. The rule now explicitly rejects `send handle.method()` on an `&self` method as a compile error. Resolves the previous §8.2/§8.3 contradiction on `Cache::get(&self, key: &K)`. Private (non-`pub`) actor methods retain their existing "references freely allowed" rule. **Handle drop semantics** (§8.2): dropping a `Handle<T>` — including the last live handle — has **no effect on actor lifetime**. Actors terminate only via runtime failure, supervisor termination, or runtime shutdown. Orphaned actors (no live handle, empty mailbox) are a known tradeoff of the non-reference-counted handle model; explicit self-termination is deferred. **Generic actor `Send` bounds** (§8.3): all type parameters on an `actor` declaration must carry `Send`, not only those used in `pub` method signatures. **Supervisor restart semantics** (new §8.7a): restart always runs a fresh `init` with the supervisor's stored construction arguments — no state preservation across restart; old `Handle<T>` values become permanently dead and are not transparently redirected to the restarted instance (callers re-fetch from the supervisor); queued mailbox messages are discarded; init failures count toward `max_restarts`; `rest_for_one` requires children in an ordered collection, with a compile-time warning and `one_for_one` fallback if dynamic. **Supervisor restart window is sliding** (§8.7): `window_secs` is a wall-clock sliding window over each restart's timestamp, not a fixed window with reset boundaries. **Mailbox-full + actor-dies race** (§8.11): blocked senders wake immediately on destination death; `send` drops silently, `send_timeout` returns new variant `SendError::Dead` (added to §8.5 `SendError`), request/reply returns `Err(ActorError::Dead)`. Wake order unspecified. Supervisor restart does not redirect blocked senders. **Re-entrant call detection** (new §8.10.1): direct self-calls (A → A via request/reply on own handle) return new variant `ActorError::SelfCall` immediately via an O(1) runtime check — variant added to the `ActorError` enum in §8.8. Multi-actor cycles (A → B → A) are documented as a hazard but not detected in v0.4.3. Fire-and-forget self-sends (`send self.handle.method(args)`) are legal and are the correct self-scheduling pattern. Cross-references §11.3 on-chain reentrancy as a distinct mechanism. **Blocking operations in handlers** (new §8.11a): actor handlers run in an async context; the stdlib is already async-only, so no new forbid is needed there. **FFI:** calling a synchronous `extern "C"` function from inside a handler (directly or transitively) is a compile error. Handler-safe FFI must be declared `extern "C" async` (§4.9) — the compiler emits an awaitable wrapper that offloads to the runtime's blocking pool. `spawn_blocking` intrinsic deferred. **Select arm fairness** (§8.6): arms are checked top-to-bottom deterministically (not round-robin), making `select` reproducible under test. **Actors forbidden in `onchain`** (§8.1, §11.1, §12.3): the `actor` keyword, `spawn`, `send`, `send_timeout`, `select`, `timeout(ms)`, `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>`, `@supervisor`, `@mailbox`, and `extern "C"`/`extern "C" async` FFI are all compile errors inside `onchain` modules. Transitive imports of actor-using native modules through pure-function boundaries remain allowed — the forbid is on *spawning inside onchain*, not on depending on actor-using code. **Deferred to a future amendment:** `init() -> Result<Self, E>`, async `init`, explicit `handle.stop()` / `stop c` intrinsic, `spawn_blocking` intrinsic, multi-actor cycle detection, `ChildSpec<T>` as a language-visible type. No grammar changes. No new keywords (still 40). |

---

*Working title: Sploosh. Name subject to change.*
*This spec is a living document. v0.4.3-draft — April 2026.*
