# SPLOOSH Language Specification v0.5.10-draft

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
7. **Spec fits in a prompt.** The language core is split into two prompt-sized artifacts — `docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md` (≈4,800 `cl100k_base` tokens; syntax, types, ownership, math, actors, errors, modules, runtime, FFI, attributes, testing, diagnostics, manifest, plus a "Common LLM Mistakes" appendix) and `docs/spec-plans/LANGUAGE_SPEC_PROMPT_WEB3.md` (≈1,500 tokens; §11 on-chain surface). Frontier context windows hit 1M+ tokens in 2026 and a combined ~6,300-token Sploosh PROMPT is well under 1% of context, so the budgets are not constrained by frontier capacity. They are constrained by **(a) attention quality** — LLMs retrieve and reason worse from sprawling prompts even when they fit; **(b) prompt portability** across the long tail of smaller / on-device / 8K-context-window models that practitioners ship to edge environments and on-chain dev tooling; and **(c) per-token economics** at ecosystem scale, where each prompt is loaded N times across every developer session. As of v0.5.9 these soft per-file targets are CI-enforced ceilings via `scripts/check_prompt_budget.py` with three-tier semantics (pass `<` 90%, warn 90–100%, fail `>` 100%); amendments that legitimately need more room must bump the ceiling in this principle with rationale rather than silently absorb the overage (precedent: v0.5.8 commit `bd26e8f` raised `_CORE` from `~4,000` to `~4,800` after the prompt split). The retired combined `LANGUAGE_SPEC_PROMPT.md` redirects readers to the split files.

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

### 2.3 Keywords (39 total)

**Declarations:**
`fn` `let` `const` `type` `struct` `enum` `trait` `impl` `mod` `use` `pub` `extern`

**Control Flow:**
`if` `else` `match` `for` `in` `while` `loop` `break` `continue` `return`

**Types & Values:**
`self` `Self` `true` `false` `as`

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

**Off-chain cost (`W0010`).** `u256` is a primitive on every target, but native and wasm
have no hardware support — arithmetic compiles to multi-instruction software emulation
(~10–50x slower than `u64`). The compiler emits warning `W0010` at any `u256` **arithmetic**
site outside `onchain` modules. Triggers: the operators `+`, `-`, `*`, `/`, `%`, `<<`,
`>>`, comparisons (`<`, `>`, `<=`, `>=`, `==`, `!=`), the explicit-overflow methods
`wrapping_*` / `saturating_*` / `checked_*` (§4.8), and the §4.10 integer methods that
perform multi-instruction work: `pow`, `isqrt`, `ilog2`, `ilog10`, `count_ones`,
`count_zeros`, `leading_zeros`, `trailing_zeros`, `rotate_left`, `rotate_right`,
`swap_bytes`, `to_be`, `to_le`, `from_be`, `from_le`. Type declarations, struct fields,
function parameters, return types, `as` casts, and literal construction do **not** fire
the warning — passing `u256` through off-chain code (e.g., chain-bridge value plumbing)
is free. The methods `abs` (no-op on unsigned), `min`, `max`, and `clamp` also do not
fire — they are comparison-based and incur no emulation cost. `W0010` is warn-by-default;
suppress at the call site or module level with `#[allow(W0010)]` if the off-chain
arithmetic is intentional. Not emitted inside `onchain` modules. See §18.2 and the
registry entry in `docs/reference/compiler-errors.md` for the canonical trigger list.

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
Shared<T>           // Refcounted immutable shared pointer
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
| `Display` | `fn to_string(&self) -> String` | Human-readable format for `{}` (also derivable; see §12.2) |
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
- For `Shared<T>` values, the wrapper itself drops in scope-reverse order as
  usual; the inner `T` drops only when the last live clone goes out of scope,
  which may be earlier or later than any individual wrapper's lifetime. The
  order remains deterministic given the set of holders — there is no GC and
  no deferred finalization. See §4.4a.

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

**No `Rc<T>` or `Arc<T>` in Sploosh.** Use `Shared<T>` (§4.4a) for shared
*immutable* data across threads and actors; use `Handle<T>` (§8.2) for shared
*mutable* state behind an actor. Use `Map<Id, T>` with integer IDs for
graph-like structures within a single actor.

### 4.4a Shared Immutable Data with `Shared<T>`

`Shared<T>` is an atomically refcounted pointer to an immutable `T`. It is
the Sploosh answer for read-only data that many actors need to see —
configs, lookup tables, parsed ML weights, interned strings, read-only
caches. Without it, the only options are clone-everything (allocation
per borrow boundary), wrap-in-an-actor (every read is a message round
trip), or pass `&T` locally (does not cross actor or thread boundaries).
None of these scale for read-heavy shared data.

`Shared<T>` is deliberately strictly less than Rust's `Arc<T>`:

- **Immutable only.** `Shared<T>` can only ever produce `&T`. There is no
  `&mut *shared`, no `get_mut`, no `make_mut`, no `try_unwrap`. The type
  cannot be used as a backdoor for shared mutable state.
- **No `Weak<T>`.** Not introduced. Because `Shared<T>` cannot be stored
  in any cell that is mutable after construction (Sploosh has no `Cell`,
  `RefCell`, `UnsafeCell`, or user-visible atomics), reference cycles
  are impossible by construction and a weak form is unnecessary.
- **Deterministic drop.** When the last `Shared<T>` clone is dropped, `T`
  is dropped and the allocation is freed. No GC, no delayed reclamation.
  Preserves the "no GC" guarantee of §3.10.

**API:**

```sploosh
// Construction — one way to do it.
let cfg: Shared<Config> = Shared::new(Config::load("app.toml")?);

// Clone is O(1) — bumps the atomic refcount, no allocation, no T::clone.
let a = cfg.clone();
let b = cfg.clone();

// Read access via deref.
let name: &str = &(*cfg).name;
let count = (*cfg).max_connections;
```

**Deref produces `&T`, never a move.** Unlike `Box<T>` (§4.4), where
`*boxed` can move the inner value out of the box for non-`Copy` types,
`*shared` on a `Shared<T>` always produces an `&T` borrow and can never
move. The refcount invariant forbids it: the inner `T` is owned jointly
by all live clones, and no single clone may claim exclusive access. In
practice this means `*shared` is only ever useful inside a reference
context (`&(*shared).field`, `(*shared).method(&args)`); assignments
like `let x = *shared;` are a compile error for non-`Copy` `T`.

**Trait surface.** `Shared<T>: Clone + Send + Sync` when `T: Send + Sync`.
If `T` is not `Send + Sync`, `Shared::new(value)` is a compile error —
`Shared<T>` exists to cross thread and actor boundaries, so requiring
its inner value to be thread-safe is an enforced invariant, not a
convention. `Drop` is implemented: decrementing the refcount to zero
drops the inner `T` and frees the allocation. `Shared<T>` is **not
`Copy`** — explicit `.clone()` preserves the cost-signal of a refcount
bump at each use site.

**Actor interop.** A `Shared<T>` is an owned value (the wrapper itself
moves; the inner data is shared via refcount bump). Passing a
`Shared<T>` to an actor's `&mut self` method via `send` therefore
satisfies the §8.2 owned-parameter rule without any exception. It is
also the idiomatic reply type for an `&self` actor method that returns
cached data — the caller bumps the refcount on receive rather than
deep-cloning the value.

```sploosh
actor Worker {
    cache: Shared<LookupTable>,
    fn init(cache: Shared<LookupTable>) -> Self { Worker { cache } }
    pub fn lookup(&self, key: &str) -> Option<u64> {
        (*self.cache).get(key)                 // &T access across the refcounted pointer
    }
}

let table = Shared::new(LookupTable::load("data.bin")?);
let w1 = spawn Worker::init(table.clone());
let w2 = spawn Worker::init(table.clone());     // both workers share one allocation
```

**`Shared<T>` does not replace `Handle<T>`.** They answer different
questions: `Shared<T>` shares *reads* of immutable data; `Handle<T>`
shares *writes* to an actor's mutable state. Pick by intent — if any
actor needs to mutate the value, wrap it in an actor and share its
`Handle<T>`; otherwise reach for `Shared<T>`.

**Not available on-chain.** `Shared<T>` is a compile error inside
`onchain` modules (§11.1, §12.3). Reference counting has no gas or
storage meaning, and every on-chain value is scoped to the transaction
frame, so no refcounted-sharing primitive is needed or well-defined.

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

The pipe operator and method-chain forms are **semantically equivalent** for
iterator expressions. This is not a special-case rule for iterators — it
follows directly from §5.6: `expr |> f(a)` lowers to `f(expr, a)`, and for
iterator adaptors `|> method(args)` lowers to `.method(args)`. Both forms
produce the same call sequence and the same value.

```sploosh
// Method chain style
let names: Vec<String> = users.iter()
    .filter(|u| u.active)
    .map(|u| u.name.clone())
    .collect();

// Pipe style — same call sequence, same result
let names: Vec<String> = users.iter()
    |> filter(|u| u.active)
    |> map(|u| u.name.clone())
    |> collect();
```

Both forms are first-class. The spec does not prefer one over the other;
choose whichever reads better in context.

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
also rejected on items in `onchain` scope. The `async` function modifier and the
`.await` operator are likewise compile errors inside `onchain` — on-chain
execution is synchronous, single-threaded, and transactional, with no runtime
scheduler for any of these to run on. Transitive imports of native modules that
internally use actors are still allowed, provided the functions called across
the `onchain` boundary do not themselves touch actor intrinsics. See §11.1 and
§12.3 for the cross-target restriction surface, and §13.0 for the per-intrinsic
context column.

### 8.1a Actor Lifecycle States

Every actor observable through a `Handle<T>` is always in one of four states:

| State | Meaning |
|---|---|
| `INITIALIZING` | `spawn` has returned a handle, but `init` has not yet produced the initial `Self`. Incoming messages queue in the mailbox. |
| `READY` | `init` has returned. The actor is processing messages from its mailbox under the normal one-handler-at-a-time rule. |
| `DRAINING` | A `handle.stop()` request has been observed (§8.2a). The actor continues to handle messages already enqueued in its mailbox in normal FIFO order, but no new messages are accepted: `send` silently drops, `send_timeout` returns `Err(SendError::Dead)`, request/reply returns `Err(ActorError::Dead)`. When the mailbox empties — or a `handle.kill()` upgrade is observed — the actor transitions to `DEAD`. |
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
handle in the program — has **no effect on the actor's lifetime**. An actor
that is reachable from no live handle and has an empty mailbox is said to be
**orphaned**. Orphaned actors are recovered by one of the following five
termination paths, never by handle-drop:

1. **Cooperative stop** — `handle.stop() -> Result<(), StopError>` requests a
   graceful drain (§8.2a).
2. **Immediate kill** — `handle.kill() -> Result<(), StopError>` aborts after
   the current handler (§8.2a).
3. **Runtime failure** — bounds, overflow, or failed `assert` (§8.8).
4. **Supervisor decision** under the applicable strategy (§8.7).
5. **Runtime shutdown** when `main()` returns (§8.11).

Non-refcounted handles are an intentional design choice: cloning a `Handle<T>`
must be free of atomic refcount traffic, and lifetime is decoupled from
reachability so that supervisors and explicit termination remain the only
sources of authority over an actor's death. `handle.stop()` is the explicit
user-controlled exit path that closes the orphan-leak gap that would otherwise
exist for non-supervised actors.

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

**`Shared<T>` and the owned-parameter rule.** A `Shared<T>` value is owned
(the wrapper itself moves; the inner data is shared via atomic refcount),
so passing a `Shared<T>` to a `&mut self` method via `send` satisfies the
rule above — the call site `.clone()`s the `Shared<T>` to bump the
refcount before the send, and the receiver owns its copy of the wrapper.
This is the idiomatic way to pass read-heavy data to an actor handler
without deep-cloning. `Shared<T>` is likewise the idiomatic reply type
for an `&self` request/reply method that returns cached data. See
§4.4a for the full `Shared<T>` surface.

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

### 8.2a Cooperative Termination (`stop` and `kill`)

Two methods on every `Handle<T>` provide explicit user-controlled termination
paths that complement the runtime-driven paths in §8.7 / §8.8 / §8.11:

```sploosh
impl<A: Actor> Handle<A> {
    pub fn stop(&self) -> Result<(), StopError>;
    pub fn kill(&self) -> Result<(), StopError>;
}
```

Both methods are `&self` — the handle is never mutated. Any clone of the handle
may stop or kill the actor; concurrent stop calls from different threads
serialize on the per-actor termination flag described below. `stop()` and
`kill()` are **method calls on a handle**, not statements — there is no `stop`
keyword (the keyword count is unchanged at 39, §2.3).

**Per-actor termination flag.** The runtime maintains a 2-bit flag per actor
with values `Running`, `StopRequested`, and `Killed`. The flag is set
out-of-band by `stop()` / `kill()` via an atomic CAS that does not consume
mailbox capacity, never blocks on backpressure, and does not interact with
the per-sender FIFO guarantees of §8.11. This is the only out-of-band signal
in the actor model.

**`stop()` semantics:**

1. CAS `Running → StopRequested`. If the flag was already `StopRequested`,
   returns `Err(StopError::AlreadyStopping)`. If the flag was `Killed` or the
   actor was already `DEAD`, returns `Err(StopError::AlreadyDead)`. Otherwise
   returns `Ok(())`. **`stop()` is valid against an `INITIALIZING` actor**:
   the CAS succeeds independently of observable lifecycle state, and the
   stop signal is *latched* — the runtime checks the flag at the boundary
   between `init` returning and the first handler dispatch and transitions
   the actor `INITIALIZING → DRAINING` at that point. New sends arriving
   while the actor is still `INITIALIZING` queue normally; once `init`
   returns and the latched flag is observed, the queued messages drain in
   FIFO order under the same `DRAINING` rules as a `READY → DRAINING`
   transition. If `init` panics, the actor transitions directly to `DEAD`
   per §8.7 and the latched flag is discarded.
2. For an actor in `READY`, the observable state transitions
   `READY → DRAINING` (§8.1a) at the moment the flag is set. For an actor
   in `INITIALIZING`, the transition is `INITIALIZING → DRAINING` at the
   moment `init` returns and the latched flag is observed (see step 1).
   Messages **already in the mailbox** continue to drain in normal FIFO
   order. **New sends are rejected** from the moment the actor is
   observably `DRAINING`: `send` silently drops, `send_timeout` returns
   `Err(SendError::Dead)`, request/reply returns `Err(ActorError::Dead)`.
   No new error variants are introduced — these match the existing
   dead-mailbox behaviour of §8.5 and §8.8 exactly.
3. After each handler completes, the runtime checks the flag. When the flag is
   `StopRequested` and the mailbox is empty, the runtime transitions the actor
   to `DEAD` and runs `Drop` on the actor's state fields per §8.7a step 1.
4. `stop()` does **not** block until the actor reaches `DEAD`. It returns as
   soon as the signal is recorded. To observe completion, the caller may
   re-call any `&self` method on the handle and rely on `Err(ActorError::Dead)`
   as the terminal observation, or hold the handle alongside a separate
   completion channel populated by the actor before its last handler returns.
5. Senders **already blocked** on the mailbox at the moment of stop wake the
   same way they would for a death (§8.11): `send` resumes with the message
   silently dropped, `send_timeout` returns `Err(SendError::Dead)`, request/
   reply returns `Err(ActorError::Dead)`. Wake order is unspecified, matching
   §8.11.
6. Messages already produced by the actor's own `.await` (network calls,
   channel reads) complete normally; the actor processes their results in the
   current handler before yielding. Sploosh does not interrupt user code
   mid-handler.

**`kill()` semantics:**

1. CAS the flag to `Killed`. If the flag was already `Killed`, returns
   `Err(StopError::AlreadyDead)`. A `kill()` while the flag is `StopRequested`
   is **valid and upgrades**: the CAS succeeds, returns `Ok(())`, and the
   actor transitions `DRAINING → DEAD` after the current handler returns —
   the remainder of the mailbox is discarded. **`kill()` is valid against an
   `INITIALIZING` actor**: the flag is latched and observed at the boundary
   between `init` returning and the first handler dispatch; the actor then
   transitions `INITIALIZING → DEAD` without ever entering `READY` or
   `DRAINING`, the mailbox is discarded, and `Drop` runs on `Self` per
   §8.7a step 1. If `init` panics before the latched flag is observed, the
   actor transitions directly to `DEAD` per §8.7 and the flag is discarded.
2. Sploosh does not interrupt user code mid-handler. The runtime allows the
   currently executing handler (including any in-flight `.await`) to run to
   completion; only after the handler returns does the runtime discard the
   mailbox and transition the actor to `DEAD`. There is no equivalent of
   POSIX `pthread_cancel`.
3. Pending messages in the discarded mailbox return `Err(ActorError::Dead)`
   to their request/reply callers and silently drop for `send`, exactly per
   §8.11.

**`StopError`:** the new error enum is defined alongside `ActorError` in §8.8.

```sploosh
@error
enum StopError {
    AlreadyStopping,    // stop() called on an actor whose flag is already StopRequested
    AlreadyDead,        // actor is already DEAD, or kill() on already-Killed
}
```

`kill()` upgrading a `StopRequested` actor returns `Ok(())`, not an error —
the upgrade is a defined operation, not a redundant one.

**Self-stop and self-kill.** A handler may call `self.handle.stop()` or
`self.handle.kill()` on a stored self-handle. The signal is observed only
**after the current handler returns** — there is no "die now" effect mid-
method. This is *not* a re-entrant call and does **not** raise
`ActorError::SelfCall` (§8.10.1): `stop()` and `kill()` are out-of-band flag
operations, not request/reply calls on the actor's mailbox.

```sploosh
actor Worker {
    self_handle: Option<Handle<Worker>>,
    job_count: u64,

    fn init() -> Self {
        Worker { self_handle: None, job_count: 0 }
    }

    pub fn set_self_handle(&mut self, h: Handle<Worker>) {
        self.self_handle = Some(h);
    }

    pub fn shut_down_when_done(&mut self) {
        if let Some(h) = &self.self_handle {
            // Returns Ok(()); the stop flag is observed after this handler exits.
            // Any messages already enqueued ahead of this one continue to drain.
            let _ = h.stop();
        }
    }
}
```

**Supervisor interaction.** A child terminated via `stop()` or `kill()` is
treated as **intentionally terminated**, not as a failure. The supervisor
does **not** restart the child under any strategy (`one_for_one`,
`one_for_all`, `rest_for_one`), and the termination does **not** count toward
`max_restarts`. This is the explicit user-controlled exit path for a
supervised child. As a side effect, if the user wants a `one_for_all` cohort
to die together when one child is intentionally stopped, they must call
`stop()` on each child themselves — supervisor-managed cascading restart does
not apply to user-driven termination. The supervisor's stored handle
becomes permanently dead exactly as in §8.7a step 3, but no fresh `init`
follows.

**Worked example.** Clean shutdown of a non-supervised actor cluster — the
orphaned-actor scenario the v0.5.4 amendment was introduced to fix:

```sploosh
fn main() -> Result<(), AppError> {
    let logger = spawn Logger::init();
    let workers: Vec<Handle<Worker>> = (0..4)
        |> map(|_| spawn Worker::init(logger.clone()))
        |> collect;

    run_workload(&workers)?;

    // Cooperative shutdown: workers drain in-flight tasks first, then logger
    // drains its log queue. No supervisor involved; no orphaned actors left.
    for w in &workers { let _ = w.stop(); }
    let _ = logger.stop();

    Ok(())
}
```

For an immediate-shutdown variant, replace the two loops with `kill()` calls.
A common pattern is `stop()` first, with a deadline; if the deadline elapses
without the actors reaching `DEAD`, escalate to `kill()`.

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
  `SendError::Dead` is also raised when the destination is in `DRAINING` state
  (§8.1a, §8.2a): a stopping actor rejects new sends from the moment its
  termination flag is set, even though the actor itself has not yet reached
  `DEAD`.

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

**Intentional termination is not a failure.** A child terminated via
`handle.stop()` or `handle.kill()` (§8.2a) is treated as **intentionally
terminated**: the supervisor does **not** restart it under any strategy,
and the termination does **not** count toward `max_restarts`. `rest_for_one`
and `one_for_all` do not cascade for user-driven termination. The
supervisor's stored handle becomes permanently dead per §8.7a step 3, but
no fresh `init` follows. Folding user-driven termination into the failure
path would conflate intent with bugs; the v0.5.4 amendment keeps them
distinct.

### 8.7a Restart Semantics

When a supervisor restarts a child under any strategy (`one_for_one`,
`one_for_all`, or `rest_for_one`), the runtime performs these steps in order:

1. **Drop the failed actor's state.** RAII runs via any `Drop` impls on state
   fields. The failed actor's mailbox is discarded (consistent with §8.8 —
   pending messages are lost). This Drop step runs identically when the cause
   of termination is `handle.stop()`, `handle.kill()` (§8.2a), runtime
   failure, or supervisor restart — the cause does not affect Drop semantics.
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
    Dead,                           // Actor has terminated, or has entered DRAINING (§8.1a)
                                    //   and is rejecting new request/reply attempts.
    Timeout,                        // Request/reply timed out
    SelfCall,                       // Direct re-entrant self-call detected (§8.10.1)
    PanicMessage { msg: String },   // What went wrong (for logging)
}

@error
enum StopError {
    AlreadyStopping,                // stop() called on an actor whose flag is
                                    //   already StopRequested (§8.2a)
    AlreadyDead,                    // actor is already DEAD, or kill() called
                                    //   on already-Killed (§8.2a)
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
*failure-driven* actor death. Cooperative termination via `handle.stop()` or
`handle.kill()` (§8.2a) is the orthogonal user-driven path; both classes converge
on the `DEAD` state and run `Drop` identically (§8.7a).

**Request/reply against a `DRAINING` actor.** An actor that has observed a
`stop()` request but has not yet emptied its mailbox is in `DRAINING` (§8.1a).
Request/reply attempts initiated *after* the stop signal return
`Err(ActorError::Dead)` immediately, even though the actor has not yet reached
`DEAD`: a draining actor has already rejected new work. Request/reply messages
already enqueued before the stop signal continue to drain in FIFO order.

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
detected** by the current runtime. Such cycles block indefinitely until an outer
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

**Self-stop and self-kill are legal and do not deadlock.** A handler may call
`self.handle.stop()` or `self.handle.kill()` on a stored self-handle (§8.2a).
These are out-of-band flag operations on the actor's termination state, not
request/reply calls on the mailbox; they do **not** raise
`ActorError::SelfCall`. The signal is observed only after the current handler
returns, so the running handler completes normally and then the actor
transitions to `DEAD` (or `DRAINING → DEAD` once the remaining mailbox is
processed, in the `stop()` case).

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
mailbox's current fill state. The same wake semantics apply when the cause of
death is a runtime failure, a supervisor decision, the completion of `DRAINING`
after `handle.stop()` (§8.2a), or a `handle.kill()` upgrade. Wake semantics by
call style:

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
intrinsic for offloading ad-hoc blocking work; currently, the only way to
invoke blocking code from a handler is via an `extern "C" async` wrapper or by
forwarding to a non-actor `spawn async` task.

### 8.12 Observability

Actor observability is a first-class spec artifact. Every running Sploosh
program carries enough runtime metadata to answer the operational
questions a developer or supervising agent inevitably asks: *how full is
this actor's mailbox, is this actor still alive, what restarted this
child and why, how many actors are live right now, and why did this
particular actor die*. The introspection surface is **always available
in every build mode** — there is no `@observable` attribute, no
debug-only gating, and no feature flag. The bookkeeping cost is paid on
every spawn (§8.12.6); the alternative — letting users discover their
program is unobservable in production — is worse than the bytes.

The surface is split into two layers by cost. **Cheap, constant-time
reads** live as direct methods on `Handle<T>` (§8.12.1). **Richer
queries** that walk runtime state live in the new `std::actor::observe`
module (§8.12.2). **Restart history** is rooted on the supervisor's
handle rather than the child's, because only `@supervisor`-decorated
actors run a restart loop in the first place (§8.12.3, cross-references
§8.7 and §8.7a). Dead actors retain a snapshot — including their cause
of death — for as long as any `Handle<T>` clone targeting them remains
live (§8.12.4); this is the **only** refcount in the actor model, and
it lives on a side-table next to the snapshot rather than on the actor
itself, leaving §8.2 handle-drop semantics unchanged.

Every method introduced in this section is a compile error inside
`onchain` modules under the existing §11.1 / §12.3 prohibition on
actor-runtime surface, restated in §8.12.7. The new intrinsics are
listed in §13.0; the new `std::actor` module entry is in §13.2.

#### 8.12.1 Handle introspection

`Handle<T>` exposes four direct methods. All four are `&self`,
infallible, and remain callable on a dead handle — they return the
last-known values rather than failing. Constant time except
`mailbox_len`, which is an atomic load.

```sploosh
impl<A: Actor> Handle<A> {
    pub fn mailbox_len(&self) -> usize;
    pub fn mailbox_capacity(&self) -> usize;
    pub fn alive(&self) -> bool;
    pub fn actor_id(&self) -> ActorId;
}
```

- `mailbox_len()` returns the current queued message count. The value
  is an atomic snapshot and may be **stale by one increment** by the
  time the caller observes it; senders racing with the read may have
  enqueued or dequeued a message in the interval between the load and
  any subsequent action. For dead actors the value is the last
  observed count (typically 0 — `DRAINING` or runtime-failure paths
  drain or discard the mailbox before the `DEAD` transition; see §8.7,
  §8.8).
- `mailbox_capacity()` returns the configured capacity — the value
  passed to `@mailbox(capacity: N)` (§12.1) or the runtime default
  (1024; see §8.11). The value never changes over an actor's lifetime.
- `alive()` returns `true` if the actor is `INITIALIZING`, `READY`, or
  `DRAINING` (§8.1a) and `false` if it is `DEAD`. A `true` reading
  does not guarantee that subsequent calls succeed — the actor may
  transition to `DEAD` between `alive()` returning and the next call —
  but a `false` reading is final.
- `actor_id()` returns the actor's `ActorId` (§8.12.5). The same
  handle (and every clone of it) always returns the same `ActorId`,
  including after the actor has died.

#### 8.12.2 The `std::actor::observe` module

Richer queries that need to walk runtime state live in
`std::actor::observe`. The module is `not onchain` (§11.1, §12.3).

```sploosh
use std::actor::observe;

let info: Option<ActorInfo> = observe::actor_info(&handle);
let live: Iter<ActorInfo>   = observe::actors();
let pool: Iter<ActorInfo>   = observe::actors().by_supervisor(&pool_handle);
let named: Iter<ActorInfo>  = observe::actors().by_name("worker");
```

- `observe::actor_info(&handle) -> Option<ActorInfo>` returns the
  full snapshot for the actor the handle targets. The value is
  `Some(...)` whenever the runtime still retains an entry for the
  actor — i.e., whenever any `Handle<T>` clone is live (§8.12.4) —
  and `None` only for stale handles whose snapshot has been gc'd.
- `observe::actors() -> Iter<ActorInfo>` enumerates every live actor
  in the runtime. Iteration order is **unspecified but deterministic
  for a given runtime instance and observation point** — two
  back-to-back calls within one runtime will yield the same order
  for the same population, but the spec does not commit to an
  ordering across runtime instances or across releases.
- `actors().by_supervisor(&sup) -> Iter<ActorInfo>` filters to actors
  whose supervisor's `ActorId` matches `sup`'s. Returns an empty
  iterator if `sup` is not a `@supervisor`-decorated actor.
- `actors().by_name(name) -> Iter<ActorInfo>` filters by `ActorInfo.name`.

`ActorInfo` is the snapshot record:

```sploosh
struct ActorInfo {
    id:                ActorId,
    name:              String,
    spawn_location:    String,        // file:line of the spawn site (best-effort)
    supervisor:        Option<ActorId>,
    lifecycle_state:   LifecycleState, // Initializing | Ready | Draining | Dead
    mailbox_len:       usize,
    mailbox_capacity:  usize,
    death_cause:       Option<DeathCause>,
}

enum LifecycleState { Initializing, Ready, Draining, Dead }
```

`name` is the unqualified type name of the actor (e.g. `"Worker"`).
`spawn_location` is best-effort — the runtime captures the call-site
file and line at `spawn` when debug info is available; otherwise the
field is `"<unknown>"`. `supervisor` is `Some(parent_id)` when the
actor was spawned from inside a `@supervisor`-decorated actor's `init`
or handler, and `None` otherwise.

#### 8.12.3 Supervisor-rooted restart history

Restart history is exposed on the **supervisor's** handle, not on the
child's. Non-supervised actors have no restart path (§8.7) and
therefore no history to expose. Three methods are added to
`Handle<S>` whenever `S` is `@supervisor`-decorated:

```sploosh
impl<S: Actor> Handle<S> {
    // Available only when S is @supervisor-decorated:
    pub fn restart_count<C: Actor>(&self, child: &Handle<C>) -> Result<u32, ObserveError>;
    pub fn restart_history<C: Actor>(&self, child: &Handle<C>) -> Result<Vec<RestartEvent>, ObserveError>;
    pub fn children(&self) -> Iter<ActorInfo>;
}

struct RestartEvent {
    timestamp_ms_since_spawn: u64,
    cause:                    DeathCause,
}

@error
enum ObserveError {
    NotASupervisedChild,    // see §18 / E1210 (reserved)
}
```

- `restart_count(child)` returns the **total** restart count for that
  child since the supervisor first spawned it — not just the count
  within the current `window_secs` sliding window (§8.7). Calling
  `restart_count` with a child that this supervisor does not supervise
  returns `Err(ObserveError::NotASupervisedChild)`.
- `restart_history(child)` returns the retained `RestartEvent`s in
  chronological order, oldest first. The retained history is **capped
  at a configurable limit (default 16)**; older events are dropped
  FIFO. The cap is tunable via the new `restart_history: N` parameter
  on `@supervisor` (§12.1). `cause` is the `DeathCause` of the
  termination that triggered the restart (§8.12.4); for `one_for_all`
  and `rest_for_one` strategies the cause is propagated from the
  failed sibling. Same `NotASupervisedChild` error path as
  `restart_count`.
- `children() -> Iter<ActorInfo>` enumerates currently-supervised
  children. Order matches the supervisor's internal child collection
  (e.g., insertion order for `Vec<Handle<T>>`), and that order is the
  same order `rest_for_one` uses (§8.7a).

A child terminated via `handle.stop()` or `handle.kill()` (§8.2a) is
**not a restart** — supervisors treat it as intentional termination
(§8.7) and do not restart the child. Such terminations therefore do
**not** appear in `restart_history`. They do appear in `children()`
(briefly, until the child reaches `DEAD`) and in `actor_info`'s
`death_cause` once dead.

#### 8.12.4 Dead-actor snapshot retention

When an actor reaches `DEAD` (any of the five termination paths in
§8.2), the runtime captures a final `ActorInfo` snapshot and stores
it in a side-table keyed by `ActorId`. The snapshot has
`lifecycle_state = Dead` and a populated `death_cause`:

```sploosh
enum DeathCause {
    RuntimeFailure { panic: String },     // bounds, overflow, assert (§8.8)
    Stopped,                              // handle.stop() drained the mailbox (§8.2a)
    Killed,                               // handle.kill() (§8.2a)
    Supervised { restart_pending: bool }, // supervisor terminated the child (§8.7)
                                          //   restart_pending: true if a restart is queued
    RuntimeShutdown,                      // main() returned (§8.11)
}
```

**Retention contract.** The snapshot is retained as long as any
`Handle<T>` clone targeting the actor remains live. This is a
**refcount-driven retention on the snapshot side-table**, not on the
actor itself. The contrast with §8.2 is deliberate: handle drop has
no effect on actor lifetime (§8.2 retains that property unchanged),
but handle drop *does* affect snapshot lifetime — once the **last**
handle clone drops, the snapshot is GC'd. This is the **only**
refcount in the actor model, and it exists specifically to give
post-mortem `observe::actor_info(handle)` a stable observation
window.

After GC, `observe::actor_info(handle)` against a stale handle
reconstructed from elsewhere (e.g., serialized and rebuilt — the
mechanism is implementation-defined and out of scope for v0.5.6)
returns `None`. While the snapshot is still retained,
`observe::actor_info(handle)` returns `Some(info)` with
`info.lifecycle_state == LifecycleState::Dead`.

**Behavior of other handle methods on a dead handle is unchanged.**
`handle.alive()` returns `false`. `send` silently drops
(§8.2). `send_timeout` returns `Err(SendError::Dead)` (§8.5).
Request/reply returns `Err(ActorError::Dead)` (§8.8). The §8.12
additions only add new observation surface; they do not alter the
existing dead-handle behavior.

#### 8.12.5 The `ActorId` type

`ActorId` is an opaque, `Copy + Eq + Hash` identifier assigned at
`spawn`. Two distinct actors never share an `ActorId`, and an
`ActorId` is never reused — even after the actor it identified has
died and its snapshot has been GC'd, that numeric value is retired.
The runtime assigns IDs monotonically within a runtime instance from
a non-zero counter; `ActorId(0)` is reserved as a sentinel and is
not a valid actor identifier.

`ActorId` is **not** `Send` across runtime instances. v0.5.6 has one
runtime per process — `main()` starts the runtime and runtime
shutdown on `main()` return ends it (§8.11). Comparing `ActorId`s
produced by different runtime instances is a compile error in any
context the compiler can see; see §18 / `E1211` (reserved). The
multi-runtime story is deferred to a future amendment.

`ActorId` is exported from the prelude (§13.1) because users see it
in every `ActorInfo` and every `RestartEvent`.

#### 8.12.6 Cost model

The observability surface is pay-always — every spawn pays
bookkeeping whether or not anything ever calls `observe::*`. The
explicit costs are:

- **Per-actor.** A registry entry of approximately 24 bytes (the
  `ActorId`, a pointer to the actor's runtime cell, and the
  supervisor's `ActorId` if any) plus an atomic `usize` mailbox
  counter. The mailbox counter is reused from the existing
  backpressure machinery (§8.11) — `mailbox_len()` reads the same
  atomic that `send` and `send_timeout` consult.
- **Per-supervised child.** A ring buffer of the last *N*
  `RestartEvent`s (default 16; ~24 bytes each → ~384 bytes per
  child). Tunable via `@supervisor(restart_history: N)`. A
  supervisor with no restart-history readers still pays this cost.
- **Per snapshot retention.** When an actor dies, an `ActorInfo`
  (~256 bytes assuming a short type name) is held until the last
  `Handle<T>` clone targeting that actor drops.

`observe::actors()` walks the registry and is **O(N_actors)** in the
total live-actor count. It is intended for diagnostics and triage,
not for hot paths. Holding an `Iter<ActorInfo>` across an `.await`
inside an actor handler is permitted but inadvisable — it pins
snapshots and observably delays GC of dead-actor entries whose only
remaining reference is the iterator.

#### 8.12.7 On-chain prohibition

Every method, type, and module introduced in §8.12 is a **compile
error inside `onchain` modules**. This is a one-line restatement of
the existing actor prohibition in §11.1 and §12.3 — `Handle<T>`
itself is already an `onchain` compile error, so all its methods are
too; `std::actor::observe` is an actor-runtime module; and
`ActorId` is an actor identifier. There is no on-chain observability
analog in v0.5.6.

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
Both are derivable via `@derive(Debug)` and `@derive(Display)`; the derived
shapes mirror each other (variant/struct name plus field-by-field rendering),
differing only in whether each field is rendered via its own `Debug` or
`Display` impl. Manual `impl Display for T` is still allowed when the
derived shape is wrong, but a type may not have both `@derive(Display)` and
a manual `impl Display` (same conflict rule as `Debug`). See §12.2 for the
full derive specification.

```sploosh
@derive(Debug, Display)
struct Point { x: f64, y: f64 }
// Display:  "Point { x: 1.0, y: 2.0 }"
// Debug:    "Point { x: 1.0, y: 2.0 }"

@derive(Debug)
struct Vector { x: f64, y: f64 }

impl Display for Vector {
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
- The `async` function modifier and the `.await` operator — on-chain functions
  must be synchronous end-to-end within a transaction.
- The `Shared<T>` refcounted pointer type (§4.4a) — reference counting has
  no gas or storage meaning, and every on-chain value is scoped to the
  transaction frame.

Transitive imports of native modules that internally use actors are still
allowed, provided the functions called across the `onchain` boundary do not
themselves touch any of the constructs above. The forbid is on *spawning
inside `onchain`*, not on depending on actor-using code through pure-function
boundaries. See §8.1 for the actor-side statement of this rule, §12.3 for the
stdlib restriction surface, and §13.0 for the per-intrinsic context column.

### 11.1a Storage Layout

On-chain contracts expose persistent state via `storage { ... }` blocks and the
`storage::get` / `storage::set` intrinsics (§13.0). The Sploosh surface is
target-neutral: fields resolve to opaque persistent locations, and `Map<K, V>`
keys resolve to derived locations whose concrete layout is determined by the
target backend. The same contract source compiles to EVM, Solana SBF, or future
ZK-EVM / alt-VM targets without the storage model leaking into user code. The
**bytes on chain** differ per target; the Sploosh source does not.

**EVM reference realization.** On the EVM target, Sploosh adopts the Solidity
storage layout verbatim so that Sploosh contracts can be read from, written
to, and upgraded alongside Solidity contracts without storage-layout
surprises. This is a load-bearing design choice: the most common deployment
pathway for a new on-chain language is coexistence with existing Solidity
infrastructure, and layout compatibility is what makes that coexistence real.

- **Struct fields.** Fields in a `storage { ... }` block occupy sequential
  32-byte slots in declaration order, starting at slot `0`. A field that would
  overflow its remaining same-slot space is promoted to a fresh slot. Within a
  slot, primitives are right-aligned (low byte at the low address) and packed
  in declaration order — matching Solidity's packing rules exactly. Example:

  ```sploosh
  storage {
      admin: Address,         // slot 0, bytes 0..20 (high 12 bytes zero)
      paused: bool,           // slot 0, byte 20
      fee_bps: u16,           // slot 0, bytes 21..23
      total_supply: u256,     // slot 1 (full slot)
      balances: Map<Address, u256>,   // slot 2 (map header; entries derived)
  }
  ```

- **`Map<K, V>`.** A `Map<K, V>` field occupies one slot as the **map's own
  slot** `s` (its contents are computed, not stored there). For a given key
  `k`, the value lives at slot `keccak256(abi.encode(k, s))`, where
  `abi.encode` is Solidity's ABI encoding of the key type padded to 32 bytes.
  `Address` keys pad to 32 bytes with 12 leading zero bytes; integer keys are
  zero-extended big-endian.

- **Nested maps.** For `Map<K1, Map<K2, V>>` with outer slot `s`, the inner
  map's slot is `keccak256(abi.encode(k1, s))`, and the final value lives at
  `keccak256(abi.encode(k2, keccak256(abi.encode(k1, s))))`. Deeper nestings
  recurse identically.

- **Dynamic types — `Vec<T>`.** `Vec<T>` stores its length (in elements) in
  its declared slot `s`; the element region begins at `keccak256(s)` and
  grows contiguously, matching Solidity's `T[]` storage encoding.

- **Dynamic types — `String`.** `String` matches Solidity's `string`
  storage encoding, which is dual-form. Short strings (UTF-8 payload
  ≤ 31 bytes) are packed **in-slot** at `s`: the low byte of `s` holds
  `length * 2` (low bit clear ⇒ short form) and the high 31 bytes carry
  the UTF-8 payload, right-padded with zeros. Long strings store
  `length * 2 + 1` in `s` (low bit set ⇒ long form) and the UTF-8 bytes
  at `keccak256(s)`, laid out identically to `Vec<u8>`. The form is
  chosen by length alone, and the Sploosh surface behavior of `String`
  is identical in either case.

- **Fixed arrays.** `[T; N]` occupies `ceil(N * size_of::<T>() / 32)` slots
  inline; they do not use `keccak256` derivation.

- **Storage is per-contract.** Distinct `onchain mod` declarations compile to
  distinct contracts with independent storage roots. A contract cannot read
  another contract's raw storage; inter-contract state is accessed only
  through cross-contract calls (§11.4, §11.4a).

**SVM target divergence.** The Solana target uses a fundamentally different
persistence model (account-based, with program-chosen account layouts and
borsh serialization) and does not derive slots via `keccak256`. The Sploosh
surface — `storage { ... }` declarations, `storage::get` / `storage::set`
intrinsics, and the `Map<K, V>` field type — remains identical, but the
concrete SVM layout (account schema, serialization format, rent accounting)
is deferred to a future amendment targeting Solana deployment. SVM contracts
authored against `storage` today should expect the final SVM layout to be a
Sploosh-defined schema over one or more program accounts, not a direct port
of the EVM slot derivation.

**Determinism and ordering.** All storage operations are deterministic: the
same sequence of `storage::get` / `storage::set` calls on the same input
produces the same bytes on chain. Sploosh does not introduce a hidden cache
or reorder writes; the order of storage effects matches source order within
each transaction. Costs for `SLOAD` / `SSTORE` on EVM are priced per the
active hard fork's gas schedule (§11.7a).

See §13.0 for the `storage::get` / `storage::set` intrinsic signatures and
§11.7a for the gas model that prices these operations.

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
| `ctx::compute_units_remaining()` | `u64` | Remaining compute units (§11.7a) |

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
cannot be called again while it is already executing. See §11.3a for the
runtime guard mechanism and its distinction from §8.10.1 actor `SelfCall`.

### 11.3a Reentrancy Guard Mechanism

Every `onchain mod` maintains a single **per-contract reentrancy flag** in
transient runtime state (not persisted across transactions). The flag is
implemented as a reserved boolean slot in the contract's runtime frame on EVM
and as an equivalent per-program transient state on SVM.

**Guard semantics:**

- On entry to any `pub` on-chain function that is **not** marked `@reentrant`,
  the runtime checks the flag. If already set, the call reverts with
  `ChainError::Reentrancy` (§11.4a) and all state changes in the current call
  frame are unwound (§11.7a).
- If the flag is clear, the runtime sets it, executes the function body, and
  clears it on return — whether the function returns `Ok`, returns `Err`, or
  reverts. The clear-on-revert rule means a revert inside a guarded function
  does not leave the guard stuck set; the flag always matches the current
  call-stack depth into the contract.
- A function marked `@reentrant` **skips both the check and the set**. The
  flag is neither consulted nor modified on entry or exit of a `@reentrant`
  function.

**`@reentrant` scope.** The attribute disables the guard for the marked
function only. Guarded sibling functions in the same contract continue to
observe and set the flag. This means a contract can expose a small number of
opt-in re-entrant entry points (e.g., a view that is safe to call recursively)
without weakening the guarantees of the rest of its surface.

**Cross-contract interaction.** If contract A's function `foo` (guarded) calls
contract B, and B calls back into A's function `bar`:

- If `bar` is **not** `@reentrant`, the call reverts with
  `ChainError::Reentrancy` — A's flag is still set from the outer `foo`.
- If `bar` **is** `@reentrant`, the call proceeds — the flag is not consulted.
  Authors who mark `bar` `@reentrant` are responsible for its safety under
  concurrent invocation of the same contract.

```sploosh
onchain mod vault {
    storage {
        balances: Map<Address, u256>,
    }

    pub fn withdraw(amount: u256) -> Result<(), VaultError> {
        // Guarded by default. A callee that calls back into any non-@reentrant
        // function of this contract reverts with ChainError::Reentrancy.
        let sender = ctx::caller();
        let bal = storage::get(&self.balances, sender).unwrap_or(0);
        if bal < amount { return Err(VaultError::Insufficient); }
        storage::set(&mut self.balances, sender, bal - amount);
        chain::call(sender, wallet::on_receive, amount)?; // safe: reentry caught
        Ok(())
    }

    @reentrant
    pub fn peek_balance(who: Address) -> u256 {
        // Opt-out: callable from a callback without the guard firing.
        storage::get(&self.balances, who).unwrap_or(0)
    }
}
```

**Gas cost.** The guard lowers to transient-storage primitives that match
its "transient runtime state" semantics: on EVM targets supporting EIP-1153
(Cancun and later), one `TLOAD` on entry and one `TSTORE` on both entry and
exit of every non-`@reentrant` `pub` function. Transient storage is cleared
automatically at transaction end and is never persisted, which makes the
flag unwind naturally on revert without an explicit journaling entry.
Pre-1153 EVM implementations may fall back to `SLOAD` / `SSTORE` on a
reserved slot with a mandatory clear-on-exit write — the VM's journaling
then provides the same unwind-on-revert guarantee. Either way, the
semantic invariant is that the flag is never observable across
transactions. Concrete numbers are not specified here because warm/cold
access pricing and refund rules change per hard fork; implementations
should consult the active EVM cost table.

**Distinct from actor `SelfCall` (§8.10.1).** The two mechanisms share the
word "reentrancy" but are different concepts at different layers. Actor
`SelfCall` is a runtime check in the scheduler that catches an actor handler
synchronously requesting a reply from its own mailbox — a deadlock condition
specific to the scheduler. The on-chain guard is a per-contract flag that
catches a cross-contract callback re-entering a guarded function — a
vulnerability class specific to the EVM call model. On-chain execution has
no actor scheduler, and actor execution has no storage slots, so the two
never overlap.

### 11.4 Cross-Contract Calls

Call signatures of foreign contracts must be declared at the caller's module
level via `extern onchain mod` (§11.4a). The `chain::call` intrinsic then
takes a declared function as its target and enforces argument types at
compile time. Calls return `Result<T, ChainError>` and `?` propagates a
callee revert.

```sploosh
extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
    pub fn transfer_from(from: Address, to: Address, amount: u256) -> Result<(), TokenError>;
}

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

### 11.4a Cross-Contract ABI and Call Semantics

Cross-contract calls in Sploosh are **statically typed at the caller** and
**dynamically dispatched on chain**. The caller declares the callee's public
interface as a compile-time interface block; the compiler generates argument
encoding and return decoding stubs; `chain::call` invokes the callee through
the active target's native call mechanism.

**`extern onchain mod` interface blocks.** A caller that wants to invoke
functions on another contract declares their signatures at module top level:

```sploosh
extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError>;
}
```

- The block contains only function *signatures* ending in `;` — no bodies.
- Signatures use `pub fn` to match how the callee would write its own public
  interface; the keyword is accepted and ignored at the extern site.
- Return types are always `Result<T, E>` (on-chain functions are total in the
  spec sense; see §11.3).
- Error types referenced in signatures (`TokenError` above) must be in scope
  on the caller side via `use` or local definition — the caller and callee
  must agree on the error enum's layout the same way they agree on argument
  types.
- `extern onchain mod` blocks are only allowed inside `offchain` code or at
  the top level of a crate containing `onchain mod` declarations. They are
  not allowed inside `extern "C"` blocks, inside actor declarations, or
  nested in function bodies.
- Declaring an `extern onchain mod` does not deploy, instantiate, or
  otherwise reference a specific on-chain address. The address is passed to
  `chain::call` at the call site.

**`chain::call` signature and semantics.** The intrinsic has the signature:

```sploosh
chain::call<Args, T, E>(target: Address, callee: ExternFn<Args, T, E>, args: Args)
    -> Result<T, ChainError>
```

- `target` is the on-chain address of the callee contract.
- `callee` is the name of a function declared in an in-scope `extern onchain mod`
  block (e.g., `token::balance_of`). The parameter is named `callee`
  rather than `fn` because `fn` is a reserved keyword.
- `args` is a single value of the argument tuple type, or the sole argument
  for a unary function.
- The return type is `Result<T, ChainError>` — distinct from the callee's
  own `Result<T, TokenError>`. `?` on the outer `Result` propagates
  `ChainError` to the caller's surrounding `Result`. If the caller wants to
  inspect the callee's domain error, it must unwrap `ChainError::Reverted`
  and decode the revert data (see below).

**Synchronous EVM execution.** On the EVM target, `chain::call` lowers to an
EVM `CALL` opcode. The caller's execution blocks until the callee returns or
reverts; all caller-side storage writes made before the call remain in
journaled state and are unwound only if the **entire transaction** reverts.
Gas is forwarded per the EVM default (all remaining gas minus the 1/64
reserve from EIP-150). Explicit per-call `#[gas_limit(N)]` on `chain::call`
is not yet supported and is deferred to v0.5.0.

**Solidity ABI as the reference encoding.** Argument and return encoding on
EVM is Solidity's ABI encoding: arguments are tuple-encoded with a
4-byte selector derived from `keccak256(signature_string)[0..4]`, where
`signature_string` is built from the Sploosh signature using Solidity
type names (`address`, `uint256`, `bool`, `bytes`, `string`, etc.). This
matches Solidity's function selector derivation exactly so that Sploosh
contracts can call and be called by Solidity contracts without a wrapper
layer. Encoding is a compiler responsibility; user code never constructs
calldata manually.

**`ChainError` enum.** The error type returned by `chain::call` is:

```sploosh
@error
pub enum ChainError {
    Reverted { data: Vec<u8> },
    OutOfGas,
    Reentrancy,
    InvalidTarget,
    DecodingError,
}
```

- `Reverted { data }` — the callee reverted. `data` is the callee's revert
  payload, bounded by the EVM `RETURNDATACOPY` semantics (the callee's final
  `RETURN` / `REVERT` buffer, capped by the gas available for return-data
  copy). The buffer is allocated in the caller's current call frame using the
  same allocation model Solidity uses for revert data; on-chain heap
  allocation for revert bytes is permitted for this type specifically.
  Authors who know the callee's error enum can decode `data` via the
  `@error`-generated decoder on that enum.
- `OutOfGas` — the callee exhausted its forwarded gas. Unlike `Reverted`,
  this variant carries no revert data. See §11.7a for transaction-wide OOG
  semantics.
- `Reentrancy` — the callee hit its reentrancy guard (§11.3a).
- `InvalidTarget` — the target address is not a contract, or the target
  contract has no function matching the called selector.
- `DecodingError` — the callee returned bytes that do not decode as the
  declared `T` (callee and caller disagree on the ABI).

`ChainError` lives at `std::chain::ChainError` and is re-exported from the
prelude (§13.1) so that `Result<T, ChainError>` signatures need no explicit
`use`. The canonical definition is here in §11.4a; the stdlib module page
(`docs/stdlib/chain.md`) and the prelude entry both point back at this
section.

**No delegatecall in v0.4.x.** Sploosh does not yet expose the EVM
`DELEGATECALL` opcode. `chain::call` always uses `CALL` semantics (callee
executes in its own storage context). A delegate-call intrinsic and its
storage-layout implications are deferred to v0.5.0.

**SVM target divergence.** Solana's cross-program invocation (CPI) model is
asynchronous at the VM layer — a program issues an invocation instruction
that the Solana runtime executes in a nested context. On SVM, `chain::call`
and `extern onchain mod` still compile, but the compiler lowers to a CPI
instruction rather than an EVM `CALL`. The user-level surface (synchronous
`Result<T, ChainError>` return, `?` propagation, argument typing) is
preserved so that the same Sploosh source can target either chain; the
on-chain ABI, selector derivation, and account-passing conventions are
SVM-specific and deferred to the Solana-targeting amendment (see §11.1a
SVM note).

**Distinct from `extern "C"` (§4.9).** Both `extern "C" { ... }` and
`extern onchain mod X { ... }` are declaration-only blocks nested under the
`extern` keyword, but their calling conventions, safety models, and error
surfaces are entirely different:

| Aspect | `extern "C"` (§4.9) | `extern onchain mod` (§11.4a) |
|---|---|---|
| Calling convention | C ABI (platform-specific) | Solidity ABI (EVM) or CPI (SVM) |
| Transport | In-process function call | On-chain transaction (EVM `CALL` / SVM CPI) |
| Safety model | Compiler-generated safe wrappers around raw C | Compile-time argument typing + runtime revert |
| Error surface | `Result<T, FfiError>` | `Result<T, ChainError>` |
| Allowed in `onchain`? | No (compile error) | Yes, and only declared here or in `offchain` |
| Allowed in handlers? | Only the `async` form (§8.11a) | N/A — handlers and on-chain never overlap |

Authors must not treat the two as interchangeable despite the syntactic
resemblance. A misuse — e.g., declaring an `extern "C"` block inside
`onchain`, or calling an `extern onchain mod` function from an off-chain
actor handler — is a compile error.

### 11.5 Events

```sploosh
onchain enum Event {
    Transfer {
        #[indexed] from: Address,
        #[indexed] to: Address,
        amount: u256,
    },
    Approval {
        #[indexed] owner: Address,
        #[indexed] spender: Address,
        amount: u256,
    },
    Deposit { sender: Address, amount: u256 },
}
```

**`#[indexed]` field marker.** On EVM, an event field marked `#[indexed]`
becomes an indexed topic in the emitted log, allowing off-chain indexers to
filter by that field cheaply. Unmarked fields are packed into the event's
data region.

- EVM allows up to **three** `#[indexed]` fields per event variant (topics
  1, 2, 3; topic 0 is reserved for the event signature hash). A variant with
  more than three `#[indexed]` fields on EVM is a compile error.
- On SVM, `#[indexed]` is accepted for source-compatibility but is a no-op
  at the Solana log record level — Solana programs emit a single data buffer
  per log entry. Off-chain indexers for SVM contracts must parse the full
  event payload.
- The marker is valid only on fields of variants inside an `onchain enum`
  declaration used with `emit`. Applying `#[indexed]` elsewhere is a compile
  error.

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

### 11.7a Gas Model

On-chain execution is metered. Every instruction executed on chain consumes
a resource — **gas** on EVM, **compute units** on SVM — that is bounded per
transaction and paid for by the transaction submitter. Sploosh exposes this
resource as a first-class concept in the type system (intrinsics and
attributes that are compile errors off-chain) but **does not redefine the
cost model of any target**. The authoritative cost tables are the ones
maintained by the host chains.

**EVM: gas.** On the EVM target:

- `ctx::gas_remaining() -> u256` returns the remaining gas at the point of
  the call. Available only inside `onchain` modules compiled for EVM; a
  compile error on SVM, native, and wasm targets.
- `#[gas_limit(N)]` on a `pub fn` is an **advisory** annotation surfaced in
  the deployed contract's ABI metadata; it does not itself cap execution at
  runtime. Runtime OOG is produced by the EVM, not by this annotation. The
  annotation is EVM-only — a compile error on SVM, native, and wasm.
- Opcode costs are those of the EVM Yellow Paper as amended by the active
  hard fork's EIPs (EIP-2929 warm/cold access pricing, EIP-3529 refund
  rules, EIP-1559 base fees, and later). Sploosh does not duplicate these
  tables in this specification; implementations consult the active EVM cost
  table at compile time and at execution time.

**SVM: compute units.** On the Solana target:

- `ctx::compute_units_remaining() -> u64` returns the remaining compute
  units. Available only inside `onchain` modules compiled for SVM; a compile
  error on EVM, native, and wasm targets.
- `#[gas_limit(N)]` is a compile error on SVM. Solana bounds compute via the
  runtime's compute budget instruction rather than a per-function directive;
  compute-budget configuration is deferred to the SVM-targeting amendment.
- Compute-unit costs are those of the Solana runtime as documented by the
  Solana Labs runtime version active on the target cluster.

**Native and wasm targets.** `ctx::gas_remaining`, `ctx::compute_units_remaining`,
and `#[gas_limit]` are all compile errors on native and wasm. Gas is an
on-chain-only concept; off-chain Sploosh code does not observe a metering
abstraction through these names. Authors who want generic metering off-chain
should build it at the application layer.

**Out-of-gas semantics.** On EVM, when execution exhausts the gas budget
mid-transaction, the EVM reverts the transaction. In Sploosh terms:

- All storage mutations made since the start of the transaction roll back.
  `storage::set` calls are journaled; on revert, the journal is discarded.
- All emitted events (`emit ...`) since transaction start are discarded.
- Any cross-contract call that was in progress when the enclosing frame ran
  out of gas returns `ChainError::OutOfGas` to its caller — provided the
  caller has enough remaining gas to handle the error surface.
- **Revert unwind is transaction-wide and is unaffected by per-function
  attributes.** `@payable`, `@reentrant`, `@inline`, and other function
  attributes do not alter revert semantics; a `@reentrant` function that is
  mid-execution when an outer OOG fires has its state unwound just like any
  other function's state.

On SVM, compute-unit exhaustion aborts the current top-level instruction,
and state changes made by that instruction (and by any CPI it issued) are
not committed. The same principles apply: revert unwind is atomic at the
transaction (or top-level instruction) boundary, and no per-function
attribute changes that.

**Interaction with the reentrancy guard.** The reentrancy flag (§11.3a) is
held in transient state and is unwound on revert along with storage. A
function whose outer call runs out of gas therefore does not leave the
contract's reentrancy flag stuck set: the unwind clears it. The same is true
for a callee that panics — unwind is driven by transaction outcome, not by
function-level attributes.

---

## 12. Attributes & Compiler Directives

### 12.1 Standard Attributes

| Attribute | Purpose |
|---|---|
| `@test` | Mark a function as a test case (§13.3.1) |
| `@property` | Mark a function as a randomized property test (§13.3.6) |
| `@derive(...)` | Auto-generate trait impls |
| `@error` | Auto-generate `From`, `Display`, `Error` for error enums |
| `@inline` | Hint to inline a function |
| `@payable` | On-chain: function accepts native tokens |
| `@reentrant` | On-chain: opt-in to reentrancy; disables the §11.3a guard for this function only. Does not alter guard state observed by other functions in the same contract. |
| `@supervisor(...)` | Mark an actor as a supervisor. Parameters: `strategy`, `max_restarts` (default 5), `window_secs` (default 60, sliding), and `restart_history: N` (default 16) — the per-child cap on the retained `RestartEvent` ring buffer surfaced by `Handle<S>.restart_history(child)` (§8.12.3). Older events are dropped FIFO. |
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
| `Display` | `Display` trait (for `{}`) — see derive shape below |
| `Clone` | `Clone` trait (deep copy) |
| `Copy` | `Copy` trait (bitwise copy, requires `Clone`) |
| `Eq` | `Eq` trait (structural equality) |
| `Hash` | `Hash` trait |
| `Serialize` | `Serialize` trait |
| `Deserialize` | `Deserialize` trait |
| `Ord` | `Ord` trait (total ordering) |

All derive macros work on structs and enums with mixed variant types.

**`@derive(Display)` shape.** The derived `Display` impl mirrors the derived
`Debug` impl: structs render as `StructName { field1: <field1 as Display>, field2: <field2 as Display> }`,
tuple structs as `StructName(<f0 as Display>, <f1 as Display>)`, unit structs
as `StructName`, and enums by variant — `VariantName` for unit variants,
`VariantName(<f0 as Display>, ...)` for tuple variants, and
`VariantName { field: <field as Display>, ... }` for struct variants. The
only difference from the derived `Debug` shape is that each field is
rendered via its own `Display` impl rather than its `Debug` impl.

**Field requirement.** Every field type must itself implement `Display` for
the derive to apply. A field whose type lacks `Display` produces a
compile-time error at the derive site (the same shape as the existing
`Debug`-without-`Debug`-on-a-field error).

**Conflict rule.** A type may have either `@derive(Display)` or a manual
`impl Display for T`, but not both — duplicate impls are a compile error.
Same rule as `Debug`. Drop the derive when the field-by-field shape is
wrong for the type (e.g., when an `Address` should print as `0x…` rather
than `Address(<bytes as Display>)`).

```sploosh
@derive(Display)
enum Token { Lit(i32), Op { sym: char } }
// "Lit(42)"   for Token::Lit(42)
// "Op { sym: + }"   for Token::Op { sym: '+' }
```

### 12.3 Compiler Directives and Conditional Compilation

```sploosh
#[target(evm)]         // Compile only for EVM target
#[target(native)]      // Compile only for native target
#[gas_limit(50000)]    // On-chain gas budget (EVM only; advisory)
#[indexed]             // Event field marker (EVM topic slot)
#[cfg(test)]           // Include only in test builds
#[cfg(debug)]          // Include only in debug builds
```

**`#[gas_limit(N)]` scope.** The directive is EVM-only and advisory: it
surfaces in the deployed contract's ABI metadata but does not itself cap
runtime execution (runtime OOG is produced by the EVM). Applying
`#[gas_limit]` on SVM, native, or wasm targets is a compile error. See §11.7a.

**`#[indexed]` scope.** The directive marks an event variant's field as an
indexed topic on EVM (up to three per variant; §11.5). It is a compile error
outside event variant fields. On SVM, `#[indexed]` is accepted for
source-compatibility but has no runtime effect.

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
`Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>` types, the
`@supervisor`, `@mailbox` attributes, and the `async` function modifier with its
`.await` operator. `extern "C"` and `extern "C" async` FFI blocks are also
forbidden on-chain (§4.9, §11.1). The `Shared<T>` refcounted pointer type
(§4.4a) is forbidden on-chain as well — reference counting has no gas or
storage meaning, and every on-chain value is scoped to the transaction
frame. On-chain execution is synchronous, single-threaded, and
transactional — there is no scheduler or shared-memory heap for any of
these constructs to run on. See §8.1 and §11.1 for the cross-references.

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
| `assert_eq(a, b)` | `fn<T: Eq + Debug>(&T, &T)` | native, wasm (test only) | Equality assertion; reports both values on failure (§13.3.3) |
| `assert_ne(a, b)` | `fn<T: Eq + Debug>(&T, &T)` | native, wasm (test only) | Inequality assertion; reports both values on failure (§13.3.3) |
| `assert_matches(v, p)` | `(value, pattern)` special form | native, wasm (test only) | Pattern-match assertion (§13.3.3) |
| `vec![a, b, c]` | `-> Vec<T>` | All | Vec literal |
| `vec![val; count]` | `(T: Clone, usize) -> Vec<T>` | All | Vec repeat |

**Concurrency intrinsics:**

| Intrinsic | Signature | Context | Purpose |
|---|---|---|---|
| `spawn expr` | `ActorInit -> Handle<T>` | not onchain | Spawn actor |
| `spawn async { block }` | `-> JoinHandle<T>` | not onchain | Spawn async task |
| `send expr` | `ActorMethod -> ()` | not onchain | Fire-and-forget message |
| `send_timeout(expr, ms)` | `-> Result<(), SendError>` | not onchain | Bounded send |
| `Handle<A>.stop()` | `fn(&Handle<A>) -> Result<(), StopError>` | not onchain | Cooperative graceful drain (§8.2a) |
| `Handle<A>.kill()` | `fn(&Handle<A>) -> Result<(), StopError>` | not onchain | Immediate termination after current handler (§8.2a) |
| `Handle<A>.mailbox_len()` | `fn(&Handle<A>) -> usize` | not onchain | Current queued message count, atomic snapshot (§8.12.1) |
| `Handle<A>.mailbox_capacity()` | `fn(&Handle<A>) -> usize` | not onchain | Configured mailbox capacity (§8.12.1) |
| `Handle<A>.alive()` | `fn(&Handle<A>) -> bool` | not onchain | True if the actor is not `DEAD` (§8.12.1) |
| `Handle<A>.actor_id()` | `fn(&Handle<A>) -> ActorId` | not onchain | Opaque per-spawn identifier (§8.12.1, §8.12.5) |
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
| `ctx::gas_remaining()` | `u256` | onchain, EVM only (compile error elsewhere; §11.7a) | Remaining gas |
| `ctx::chain_id()` | `u256` | onchain, EVM | Chain ID |
| `ctx::lamports()` | `u64` | onchain, SVM | SOL sent |
| `ctx::program_id()` | `Address` | onchain, SVM | Program address |
| `ctx::signer()` | `Address` | onchain, SVM | Transaction signer |
| `ctx::compute_units_remaining()` | `u64` | onchain, SVM only (compile error elsewhere; §11.7a) | Remaining compute units |
| `storage::get(field, key)` | Varies | onchain (§11.1a for layout) | Read persistent state |
| `storage::set(field, key, val)` | `()` | onchain (§11.1a for layout) | Write persistent state |
| `chain::call(addr, callee, args)` | `Result<T, ChainError>` | onchain (§11.4a for ABI) | Cross-contract call |

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
- `assert_eq`, `assert_ne`, and `assert_matches` are **test-only** intrinsics — calling them outside a `@test`-annotated function or `#[cfg(test)]` module is a compile error (`E1410`, reserved). Inside tests, their failure semantics are identical to `assert()` (panic, observed by the per-test isolation actor — §13.3.4).
- The optimizer may fuse adjacent `.sin()` and `.cos()` calls on the same input into a single `llvm.sincos` call. Math calls inside loops are auto-vectorized when the target has a SIMD libm (SVML on Intel, libmvec on glibc).
- Constant expressions involving math intrinsics are folded at compile time: `(0.0f64).sin()` becomes `0.0` during codegen, with no runtime call.

### 13.1 Prelude (auto-imported)

```
Option, Some, None
Result, Ok, Err
String, Vec, Map, Set, Box, Shared
print, format, assert
Display, Debug, Clone, Copy, Eq, Hash, Ord
From, Into, TryFrom, TryInto
Drop, Error
Fn, FnMut, FnOnce
Iter, FromIter
Send, Sync
Handle, JoinHandle, ActorId
Channel, Sender, Receiver
Address, u256
ChainError
FpCategory
```

**Test-only prelude additions** (auto-imported only under `#[cfg(test)]`,
i.e., during `sploosh test`; see §13.3):

```
assert_eq, assert_ne, assert_matches
TestFailure, Gen, Rng
```

Referencing any of these outside a `@test`-annotated function or
`#[cfg(test)]` module is a compile error (`E1411`, reserved).

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
| `std::actor`   | Actor observability and introspection (§8.12) | native, wasm |
| `std::web`     | HTTP server, routing, middleware         | native, wasm |
| `std::db`      | Database connection, query builder       | native |
| `std::chain`   | `chain::call`, `ChainError`, `extern onchain mod` (§11.4a) | all |

### 13.3 Testing

The Sploosh test framework is a first-class spec artifact, not a library
add-on. `@test` is the attribute, `std::test` is the runtime surface, and
`sploosh test` is the runner. This section specifies all three together
because they are not separable — a model writing tests must know the
attribute, the assertions, the failure semantics, and the runner contract
all at once.

`std::test` is a **compile error inside `onchain` modules** (§11.1, §12.3).
On-chain code is tested off-chain by spawning a simulated execution context;
the `@onchain_test` shape is deferred to a future amendment.

#### 13.3.1 The `@test` attribute

```sploosh
@test
fn add_works() {
    assert_eq(2 + 3, 5);
}
```

**Function shape requirements.** A `@test`-annotated function must:

1. Take **zero parameters** unless it is `@property` (see §13.3.6).
2. Return `()` or `Result<(), TestFailure>`.
3. Be a **free function** at module scope. `@test` on an associated function,
   trait method, or actor handler is a compile error.
4. Be `pub` or private — visibility does not affect discovery. The runner
   discovers tests by attribute, not by name or path.
5. Optionally be `async`. An `async @test fn` runs on a fresh per-test
   runtime (§13.3.5).

`@test` is **only honored when `#[cfg(test)]` is true** — i.e., during
`sploosh test`. In other build modes, `@test`-annotated functions are
removed by dead-code elimination after type-checking; they do not appear
in the produced binary, do not contribute to binary size, and may
reference `#[cfg(test)]`-only code.

**Naming convention.** Test functions are conventionally named
`test_<thing>` or `<thing>_works` / `<thing>_rejects_<x>`. The compiler
does not enforce a convention, but the test runner orders output
alphabetically by fully-qualified path.

#### 13.3.2 Test discovery and layout

Tests live in two locations:

- **Unit tests** — inline in the module they exercise, conventionally
  inside a `#[cfg(test)] mod tests { ... }` block. They have access to
  the parent module's private items.
- **Integration tests** — files under `tests/` at the package root. Each
  `tests/*.sp` file is compiled as its own crate root and only sees the
  package's `pub` surface. `tests/` is implicitly `#[cfg(test)]` — every
  file inside is included only by `sploosh test`.

```
my_pkg/
├── sploosh.toml
├── src/
│   ├── lib.sp
│   └── auth.sp           # contains `#[cfg(test)] mod tests { ... }`
└── tests/
    └── login_flow.sp     # integration test crate
```

**Doc tests are deferred** to a future amendment. The `@test` attribute is
the only test-bearing surface in v0.5.5.

#### 13.3.3 Assertions

The test framework uses three intrinsics that complement the existing
§13.0 `assert(cond, msg)`. All three are available **only inside
`@test`-annotated functions and `#[cfg(test)]` modules** — calling them
from production code is a compile error (`E1410`, reserved). They lower
to `panic` on failure, which the runner observes via the per-test
isolation actor (§13.3.4):

| Intrinsic               | Signature                                       | Purpose                                       |
|-------------------------|-------------------------------------------------|-----------------------------------------------|
| `assert_eq(a, b)`       | `fn<T: Eq + Debug>(&T, &T)`                     | Assert `a == b`; failure reports both values  |
| `assert_ne(a, b)`       | `fn<T: Eq + Debug>(&T, &T)`                     | Assert `a != b`; failure reports both values  |
| `assert_matches(v, p)`  | Special syntax — `p` is a §5.2 match pattern    | Assert `v` matches pattern `p`                |

`assert_eq` and `assert_ne` borrow their operands (`&T` internally) so
they do not consume non-`Copy` values. `assert_matches` uses the §5.2
match-binding rules: pattern variables introduced inside `p` are not
available after the assertion (the macro discards them). Failure
messages are produced by `Debug`, not `Display` — every type that
participates in an assertion must therefore satisfy `Debug` (typically
via `@derive(Debug)`).

```sploosh
@test
fn parses_expected_shape() {
    let result = parse("3 + 4");
    assert_matches(result, Ok(Expr::Add(_, _)));
    assert_eq(result.unwrap().to_string(), "(3 + 4)");
}
```

`assert(cond, msg)` (already in §13.0) remains the universal fallback
for predicates that are not equality- or pattern-shaped. Inside
`@test`-annotated functions it has the same panic-and-report semantics
as the test-only assertions.

**`?` interaction.** When a `@test fn` is declared `-> Result<(),
TestFailure>`, the body may use `?` to propagate errors from setup code.
The test framework reports a propagated `Err` as a test failure
(distinct from an assertion failure but indistinguishable to the runner
exit code). This is the recommended shape for any test that involves
fallible setup (`fs::read`, `net::connect`, etc.).

```sploosh
@test
fn loads_config() -> Result<(), TestFailure> {
    let cfg = Config::load("test.toml")?;   // fails the test if Err
    assert_eq(cfg.name, "test");
    Ok(())
}
```

`TestFailure` is a library type defined in `std::test`; it implements
`From<E>` for every `E: Error`, making `?` propagation transparent.

#### 13.3.4 Failure semantics and per-test isolation

Each test runs **inside its own runtime-spawned isolation actor**. The
runner spawns the actor with a one-shot completion channel, sends a
single `run` message, and observes one of three outcomes:

1. **`Ok(())`** — the handler returned normally, including
   `Ok(())`-returning `Result<(), TestFailure>` shapes. The test passes.
2. **`Err(TestFailure)`** — the handler returned `Err`. The test fails;
   the runner records the `TestFailure` payload.
3. **Actor death** — the handler panicked (failed `assert*`, bounds
   check, overflow, etc.). The runner observes
   `Err(ActorError::Dead { panic: Some(msg) })` on the completion
   channel and records the panic message as the failure cause.

Per-test isolation means a single failing test never aborts the runner.
The supervisor strategy for the test cohort is conceptually
`one_for_one` with `max_restarts: 0` — a failed test is recorded, not
restarted. The runner does not invoke `@supervisor`-decorated user
code; supervisor strategies are unrelated to the test runner's own
supervision.

**Actor lifecycle inside tests.** Tests that `spawn` actors of their own
must clean up via `handle.stop()` / `handle.kill()` (§8.2a) or rely on
the runtime-shutdown path: when the per-test isolation actor reaches
`DEAD`, every actor it spawned that has the test's runtime as its only
keepalive is terminated as part of the runtime-shutdown sweep. Tests
that share a runtime via `--test-threads=1` must clean up explicitly —
a leaked spawn from one test is observable by the next.

#### 13.3.5 Async and actor tests

`async @test fn ...` is permitted. The runner spawns a fresh runtime
per test and drives the future to completion under the same isolation
actor. `.await` works exactly as in production code; channels, select,
and timeouts are all available.

```sploosh
@test
async fn fetches_payload() -> Result<(), TestFailure> {
    let body = http::get("http://localhost:8080/health").await?;
    assert_eq(body, "ok");
    Ok(())
}
```

A test that spawns its own actor system follows the same pattern; the
test owns the supervisor handle and either lets it die at test end or
calls `.stop()` explicitly.

```sploosh
@test
fn counter_increments() {
    let counter = spawn Counter::init(0);
    send counter.inc(5);
    send counter.inc(3);
    assert_eq(counter.get(), 8);
    let _ = counter.stop();
}
```

#### 13.3.6 Property tests

`@property` is a sibling attribute to `@test` for randomized testing.
A `@property fn` takes one or more parameters of types that implement
`Gen`; the runner generates `N` cases (default 256), shrinks failures
to a minimum reproducer, and reports both the original failing input
and the shrunk minimum.

```sploosh
@property
fn reverse_reverse_is_identity(v: Vec<i32>) {
    assert_eq(v.iter().rev().rev().collect::<Vec<i32>>(), v);
}
```

**`Gen<T>` trait.** A type participates in property generation by
implementing `std::test::Gen`:

```sploosh
trait Gen {
    type Item;
    fn generate(rng: &mut Rng, size: u32) -> Self::Item;
    fn shrink(value: Self::Item) -> Iter<Self::Item>;
}
```

`size` is a 0–`size_max` complexity bound the runner increases as it
explores; `shrink` returns an iterator of strictly-smaller candidates
the runner tries on a failed input. `Gen` impls are provided by the
prelude for: every primitive integer type, `bool`, `f32`/`f64`, `char`,
`String`, `Vec<T: Gen>`, `Option<T: Gen>`, `Result<T: Gen, E: Gen>`,
and tuples up to arity 12.

**Runner contract.** Failing inputs are reported with their RNG seed,
case index, and shrunk minimum. The same seed reproduces the same
shrunk minimum byte-for-byte (deterministic shrinking — implementations
must use a deterministic shrinking schedule). Property failures use the
same `TestFailure` reporting channel as `@test` failures; the runner
adds the seed and shrink trace to the failure record.

**CLI control.** `sploosh test --cases=N` overrides the default 256.
`sploosh test --seed=0xCAFEBABE` fixes the seed for reproduction. Both
flags are advisory (a property test may opt out via
`@property(cases: M)`).

#### 13.3.7 The `sploosh test` runner

`sploosh test` is the canonical runner CLI. Its full surface lives in
`docs/tooling/build-system.md`; the spec-level contract is:

| Flag                          | Default | Purpose                                                         |
|-------------------------------|---------|-----------------------------------------------------------------|
| `--filter <pat>`              | none    | Only run tests whose fully-qualified path matches `<pat>`       |
| `--exact`                     | off     | `--filter` is an exact match instead of a substring             |
| `--test-threads <N>`          | core count | Run `N` tests concurrently (1 disables parallelism)          |
| `--nocapture`                 | off     | Forward test stdout/stderr to the terminal during the run      |
| `--seed <hex>`                | random  | Fix the property-test RNG seed for reproduction                |
| `--cases <N>`                 | 256     | Override the per-property case count                           |
| `--format <human\|json>`      | human   | Match `--error-format` (§18.5) — JSON is one event per line     |

**Determinism.** With `--test-threads=1 --seed=<fixed>`, two runs of the
same source against the same compiler version produce byte-identical
output. This is the contract LLM agents and CI snapshot tests rely on;
implementations must not introduce non-deterministic ordering, timing,
or formatting under those flags.

**Exit codes.** `0` if all tests pass; `1` if any test fails;
`2` for runner errors (build failure, no tests found when a filter
was specified, etc.).

#### 13.3.8 What `std::test` exposes

The `std::test` module is the assertion+property API surface. Its
public items are:

- `TestFailure` — failure record returned by `Result<(), TestFailure>`
  shaped tests. Constructable via `TestFailure::new(msg: String)` and
  via `From<E>` for every `E: Error`.
- `Gen` — trait, see §13.3.6.
- `Rng` — opaque deterministic random source passed to `Gen::generate`.
  Methods: `next_u32`, `next_u64`, `gen_range(min, max)`, `shuffle(&mut [T])`.
- Re-exports of `assert`, `assert_eq`, `assert_ne`, `assert_matches` for
  documentation locality; the prelude already imports them.

All of these are **`#[cfg(test)]`-only** — referencing them outside a
test build is a compile error (`E1411`, reserved).

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

### 14.1 Project Manifest (`sploosh.toml`)

The project manifest is the single source of truth for package identity,
dependency graph, build configuration, and target selection. Every Sploosh
package is rooted at a `sploosh.toml`. The toolchain mirror of this section
lives in `docs/tooling/sploosh-toml.md` — the spec is authoritative; the
tooling page restates it.

A minimal single-package manifest:

```toml
[project]
name = "my-app"
version = "0.1.0"
edition = "0.5"

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

#### 14.1.1 `[project]` table

| Field | Type | Required | Meaning |
|---|---|---|---|
| `name` | string | yes | Package name. ASCII identifier characters plus `-` and `_`; must not start with a digit. |
| `version` | string | yes | Semantic version (`MAJOR.MINOR.PATCH`, optionally with pre-release / build metadata per SemVer 2.0). |
| `edition` | string | yes | Sploosh language edition (the language version, not a year). Allowed values track the spec's released minor versions: `"0.5"` is the v0.5.x edition. Edition strings are stable identifiers; renaming is a breaking change. |
| `description` | string | no | One-line summary for registries. |
| `license` | string | no | SPDX license expression (e.g., `"Apache-2.0 OR MIT"`). |
| `authors` | list of strings | no | Contributor identities (e.g., `"Name <email>"`). |
| `repository` | string | no | URL of the source repository. |

Unknown fields in `[project]` are a hard error — the manifest is a contract,
not a hint surface.

#### 14.1.2 Dependency tables

Three tables share the same shape:

- `[dependencies]` — required to build the package's library / binary.
- `[dev-dependencies]` — required only for `cfg(test)` builds and for
  `tests/` integration crates. Not visible from non-test code; not
  forwarded to dependents.
- `[build-dependencies]` — reserved for future build-script support; in
  v0.5.3 the section is parsed and validated but no build-script
  invocation is specified yet.

Each entry is either a version string or an inline table:

```toml
[dependencies]
sploosh_web   = "0.3"
sploosh_chain = { version = "0.2", features = ["evm"] }
sploosh_proto = { git = "https://github.com/example/sploosh_proto", rev = "a1b2c3d" }
local_helper  = { path = "../local_helper" }
```

Inline-table fields:

| Field | Type | Meaning |
|---|---|---|
| `version` | string | SemVer requirement. Defaults to `"*"` if absent and another source field is present. |
| `features` | list of strings | Features to enable on this dependency. |
| `default-features` | bool | Disable the dependency's `default` feature group when `false`. Defaults to `true`. |
| `optional` | bool | When `true`, the dependency is only linked if a `[features]` entry activates it via the `"dep:foo"` or `"foo/feat"` syntax. |
| `git` | string | Git source URL. Mutually exclusive with `path`. |
| `rev` | string | Required when `git` is set. **Branches and tags are not allowed** as floating refs; pin to a commit SHA for reproducibility. |
| `path` | string | Workspace-internal filesystem source. Mutually exclusive with `git`. Path must resolve inside the same workspace as the consuming package. |

Dependency source precedence: `path` > `git` > registry. Setting more than
one source on a single entry is a manifest error.

#### 14.1.3 `[features]` table

Features are additive sets of conditional-compilation flags. The grammar:

```toml
[features]
default = ["json"]                   # implicitly enabled feature group
json = []                            # leaf feature, no transitive activation
postgres = ["sploosh_db"]            # activates an optional dependency
analytics = ["sploosh_db/metrics"]   # activates a feature on a dependency
audit = ["dep:sploosh_audit"]        # explicit optional-dep activation form
```

- `"name"` activates the local feature `name`.
- `"crate/feature"` activates `feature` on dependency `crate`.
- `"dep:crate"` activates the optional dependency `crate` without enabling
  any same-named local feature (resolves the Cargo-2018 ambiguity by being
  explicit).

A feature listed in `default = [...]` is enabled unless the consumer sets
`default-features = false`. `cfg(feature = "name")` (§12) is the only
in-source way to test feature state.

#### 14.1.4 `[target.<target>.dependencies]` tables

Per-target dependency overrides use one section per target. Recognized
target names are the four build targets: `native`, `wasm`, `evm`, `svm`.

```toml
[target.wasm.dependencies]
sploosh_web = { version = "0.3", default-features = false, features = ["client"] }

[target.evm.dependencies]
sploosh_chain = { version = "0.2", features = ["evm"] }
```

The target-conditional sections are merged additively with the base
`[dependencies]` table at resolution time. A dep declared in both base and
a target section must agree on `version`/`source`; only `features` and
`default-features` may differ. The corresponding `[target.<target>.dev-dependencies]`
and `[target.<target>.build-dependencies]` sections are accepted with the
same merge rule.

Compile-time on-chain prohibitions (§11.1, §12.3) still apply: a dependency
made available under `[target.evm.dependencies]` does not bypass the
on-chain stdlib restrictions.

#### 14.1.5 `[targets]` table

The `[targets]` table is the *project-level* default target configuration.
It is distinct from `[target.<target>.dependencies]` (§14.1.4): one declares
which targets a project supports and which is the default; the other
declares per-target dependency variations.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `default` | string | `"native"` | Target used when `sploosh build` runs without `--target`. Must be one of `native`, `wasm`, `evm`, `svm`. |
| `contracts` | list of strings | `[]` | The on-chain target set the project deploys to. Subset of `["evm", "svm"]`. |

#### 14.1.6 `[profile.<name>]` tables

Profiles select compiler and linker behaviour for a build. Four built-in
profiles are predefined and may be customized; additional profiles can be
declared via `inherits`.

| Profile | Used by | Default `inherits` |
|---|---|---|
| `dev` | `sploosh build` (no `--release`) | — (built-in defaults) |
| `release` | `sploosh build --release` | — (built-in defaults) |
| `test` | `sploosh test` (also `cfg(test)` paths in `dev`) | `dev` |
| `bench` | `sploosh test --bench` | `release` |

Profile knobs:

| Knob | Type | `dev` default | `release` default | Meaning |
|---|---|---|---|---|
| `opt-level` | `0`–`3`, `"s"`, `"z"` | `0` | `3` | Optimization level. `"s"` optimizes for binary size, `"z"` for size more aggressively. |
| `lto` | `false`, `"thin"`, `"fat"` | `false` | `"thin"` | Link-time optimization. `"thin"` is parallel ThinLTO; `"fat"` is whole-program LTO. |
| `debug` | `0`, `1`, `2`, `false` | `2` | `0` | Debug info level. `0` / `false` = none, `1` = line tables, `2` = full. |
| `strip` | `"none"`, `"debuginfo"`, `"symbols"` | `"none"` | `"debuginfo"` | Symbol stripping policy applied at link. |
| `incremental` | bool | `true` | `false` | Enable incremental compilation cache. |
| `overflow-checks` | bool | `true` | `true` | Insert checked-arithmetic guards (§4.8). **Frozen `true` for `evm` and `svm` targets — overrides any user setting and emits warning `W0xxx`.** |

Custom profiles inherit from one of the built-ins (or another custom
profile, transitively):

```toml
[profile.release-small]
inherits = "release"
opt-level = "z"
lto = "fat"
strip = "symbols"
```

The compiler does not expose `codegen-units` (LLVM-specific implementation
detail) or a `panic = "abort"|"unwind"` choice (Sploosh's failure model is
fixed by §4.8 / §8 — there is no unwind path to choose).

Per-target profile overrides (e.g., `[profile.release.evm]`) are not
permitted in v0.5.3. Use `#[cfg(target = "evm")]` (§12.3) and feature flags
for target-specific code paths instead. The reservation is left open for a
future amendment if real demand emerges.

#### 14.1.7 `[runtime]` table

Native/wasm runtime tunables. All fields optional.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `threads` | integer ≥ 1 | one per CPU core | M:N scheduler thread count (§8.10). |
| `mailbox_default_capacity` | integer ≥ 1 | `1024` | Default mailbox capacity for actors that do not specify `@mailbox(capacity: N)` (§8.5, §8.10). |

The `[runtime]` table is silently ignored when building `evm` or `svm`
targets — there is no Sploosh-level runtime on-chain.

#### 14.1.8 Resolution semantics

- **Version requirement syntax** matches Cargo: caret (`"^0.3"` ≡ `"0.3"`),
  tilde (`"~0.3.4"`), exact (`"=1.0.0"`), comparison (`">=0.3, <0.5"`),
  wildcard (`"0.3.*"`).
- **Resolver**: Sploosh uses *resolver v2* unification semantics. Features
  enabled for a dependency in one target/dev-dep context do not leak into
  other contexts; specifically, dev-dependency features are not unified
  with non-test feature graphs.
- **Conflict detection** is structural: incompatible version requirements
  on the same dependency in the resolved graph are a manifest-resolution
  error, never silently coalesced.
- **`edition` is package-scoped** — the consuming package's edition
  determines its own compilation rules; dependencies compile under their
  own edition. There is no cross-edition feature gate in v0.5.3 because
  v0.5 is the only released edition.

### 14.2 Workspaces

A workspace is a set of packages built from a single dependency graph and
sharing one `sploosh.lock`. Workspaces let monorepos pin one resolved
version of every transitive dependency across all members.

The root manifest of a workspace contains a `[workspace]` table and **no
`[project]` table** — the root is not itself a buildable package:

```toml
# Root sploosh.toml
[workspace]
members = ["crates/*", "contracts/token"]
exclude = ["crates/scratch"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "0.5"
license = "Apache-2.0"

[workspace.dependencies]
sploosh_web   = "0.3"
sploosh_chain = "0.2"
```

| Section | Meaning |
|---|---|
| `[workspace]` | Marker that this manifest is a workspace root. |
| `members` | List of relative paths (globs allowed) identifying member packages. |
| `exclude` | List of paths under `members` globs that should be skipped. |
| `resolver` | Always `"2"` in v0.5.3. The field is required so future resolver versions are an explicit opt-in. |
| `[workspace.package]` | Default values for member `[project]` fields. |
| `[workspace.dependencies]` | Pre-resolved version requirements that members may inherit by reference. |

Members consume workspace defaults via the `.workspace = true` form:

```toml
# crates/api/sploosh.toml
[project]
name = "api"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
sploosh_web.workspace = true

[target.wasm.dependencies]
sploosh_chain.workspace = true
```

Inherited values may be locally overridden field-by-field, but enabling
extra features on a workspace-inherited dependency is done with the
inline-table form: `sploosh_chain = { workspace = true, features = ["svm"] }`.

A workspace has exactly one `sploosh.lock` (§14.3) at the workspace root.
Member-level lockfiles are not permitted.

### 14.3 Lockfile (`sploosh.lock`)

The lockfile records the exact resolved dependency graph for a manifest
or workspace. It is **checked into version control for binaries and
workspaces** and is otherwise generated and refreshed as needed for
libraries.

The lockfile is TOML with one `[[package]]` array entry per resolved
package:

```toml
version = 1

[[package]]
name = "sploosh_web"
version = "0.3.2"
source = "registry+https://packages.sploosh.dev"
checksum = "blake3:K6Y2QF3RZBXWNYV2T3X6UQEI5JJ4J6S7NTWWF7PADGUZB6E5W2KQ"
dependencies = ["sploosh_proto"]

[[package]]
name = "sploosh_proto"
version = "1.0.4"
source = "git+https://github.com/example/sploosh_proto?rev=a1b2c3d#a1b2c3d4e5f6..."
checksum = "blake3:H7TXP7DFA6L2B4ZA3V42M77N3MN6S55F6TZTRQAPQYP3XQXHQGUA"
dependencies = []
```

Entry fields:

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Package name. |
| `version` | string | Resolved SemVer. |
| `source` | string | Source URL with discriminator (`registry+...`, `git+...`, `path+...`). |
| `checksum` | string | `"blake3:<base32>"`. The hash is **Blake3** of the package's source archive (registry packages) or of the resolved git tree (`git` deps). 32 raw bytes encoded as RFC 4648 base32 without padding. |
| `dependencies` | list of strings | Names of direct dependencies as resolved. |

Determinism requirements:

- Entries are ordered alphabetically by `name`, then by `version`.
- TOML serialization uses LF line endings and no trailing whitespace.
- The top-level `version` field is `1` for the v0.5.3 lockfile schema. A
  schema bump increments this integer; tools must refuse unknown values.

Update semantics:

- `sploosh build`, `sploosh test`, `sploosh check` *verify* the lockfile
  against the manifest. If the manifest contains a dependency not satisfied
  by the lockfile, the build fails with diagnostic `E14xx` (reservation
  only — no registry entry assigned in v0.5.3, per §18.4 / Growth policy);
  the user must run `sploosh update`. These commands never write to
  `sploosh.lock`.
- `sploosh update` is the **only** command that may rewrite `sploosh.lock`.
  Without arguments it refreshes the entire graph; `sploosh update <name>`
  refreshes a single package and its transitive dependencies.
- A workspace lockfile applies to every member; per-member lockfiles are
  rejected as a workspace error.

### 14.4 Dependency sources

Three source kinds are valid in v0.5.3:

| Source | Form | Reproducibility |
|---|---|---|
| Registry | `name = "0.3"` (default) | Resolved version + Blake3 checksum recorded in the lockfile. |
| Git | `{ git = "...", rev = "<commit-sha>" }` | `rev` is required and must be a commit SHA. Branch and tag refs are rejected as non-reproducible. |
| Path | `{ path = "../local" }` | Workspace-internal only. Path dependencies that escape the workspace are a manifest error. |

The default registry endpoint (URL, authentication, publishing flow) is
**deferred to v0.6+** — registries are out of scope for v0.5.3, which
specifies only the manifest and lockfile contract. Registry sources are
parsed and resolved against the local filesystem mirror until the
registry surface lands.

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

extern_block   = "extern" extern_target "{" { extern_fn } "}" ;
extern_target  = STRING_LIT | "onchain" "mod" IDENT ;
extern_fn      = [ "pub" ] "fn" IDENT "(" params ")" [ "->" type ] ";" ;

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
               | "true" | "false" ;
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
| Diagnostic format + stable error codes | Machine-actionable compiler output is the AI-native lever. Stable `E<NNNN>` codes, rustc-compatible applicability vocabulary, and an NDJSON mode let LLM agents round-trip fix-and-retry loops deterministically. |
| `Shared<T>` immutable refcount primitive | Chosen over `Arc<T>` to eliminate interior-mutability pairing and cycle risk by construction. Split from `Handle<T>` by intent: reads → `Shared<T>`, writes → actor + `Handle<T>`. Strict `T: Send + Sync` requirement keeps the LLM surface narrow. |
| Manifest: Cargo-shaped, four built-in profiles, Blake3 lockfile, edition = language version | The compiler trips on the manifest first; cheaper to spec it once, exactly, than to retrofit. Cargo-exact profiles (`dev`/`release`/`test`/`bench`) maximize Rust-trained-model recall. Blake3 is already present in `std::crypto` on every target and is faster than SHA-256 for typical lockfile sizes. `[target.X.dependencies]` sections group target-specific deps spatially rather than scattering `targets = [...]` flags inline. `edition = "0.5"` ties the language edition to the shipped spec version — pre-1.0 cadence is the language, not the calendar. `codegen-units` and `panic` are deliberately omitted: the former leaks LLVM, the latter has no choice to make under §4.8's fixed failure model. |
| Cooperative termination via `handle.stop()` / `handle.kill()` | Method-form (§8.2a) rather than the `stop c` keyword form considered in v0.4.3 — keeps the keyword count at 39 and avoids any grammar change. Handle-drop continues to *not* kill the actor: non-refcounted handles are an intentional design choice (no atomic refcount on every clone), and explicit termination provides the path the original model was missing. Two methods rather than one (`stop()` graceful drain + `kill()` immediate) because the use cases are genuinely different — graceful is the OTP default, but immediate is needed for shedding a buggy or runaway actor without waiting for its mailbox to drain. Supervisor sees `stop`/`kill` as **intentional termination, not failure**: folding it into `max_restarts` would conflate user intent with bugs. The `Result<(), StopError>` return type follows the §6.1 "no exceptions, every fallible operation is `Result`" rule even though the only failure modes are already-dead and already-stopping. Both methods are `&self` because the handle is never mutated; multiple clones racing to stop the actor serialize on the per-actor termination flag, which is set out-of-band and never blocks on mailbox backpressure. |
| Actor observability (`std::actor::observe` + handle introspection + supervisor restart history) is a first-class spec artifact, always-on in every build mode | The runtime needs an answer to *what is this actor doing right now* before the compiler exists; deferring observability would force every implementer to invent the same surface differently and would let users discover their program is unobservable in production. **Hybrid placement** (cheap reads on `Handle<T>`, richer queries in `std::actor::observe`) keeps `mailbox_len` / `mailbox_capacity` / `alive` / `actor_id` constant-time and discoverable on the type users already hold, while reserving the registry-walking surface for an explicit module so users opt in by importing it. **Restart history rooted on the supervisor's handle** rather than the child's because only `@supervisor`-decorated actors run a restart loop — non-supervised actors have no restart path (§8.7) and therefore no history to expose. **Global runtime registry** (BEAM `:observer` shape) for `observe::actors()` because answering "which actors are pinning memory right now" is the operational question users ask first; the per-actor registry entry is paid anyway for the existing supervisor and mailbox machinery. **Last-known dead-actor snapshot retained until last handle drops** — this is the only refcount in the actor model, and it lives on a side-table next to the snapshot rather than on the actor itself, leaving §8.2 handle-drop semantics unchanged. The contrast was called out explicitly in §8.12.4 to prevent future readers from concluding `Handle<T>` is now refcounted. **Always-available, all-build-modes** rather than a `@observable` attribute or a debug-only feature flag because conditional observability fails the moment it is most needed (production triage), and the bookkeeping cost (~24 bytes registry + atomic mailbox counter + ~384 bytes per supervised child) is dwarfed by an actor's own footprint. **`@supervisor(restart_history: N)` extends an existing attribute** rather than introducing a new one — the keyword count stays at 39 and the grammar is unchanged. **Two new diagnostic registry slots reserved** (`E1210` non-supervised-child, `E1211` ActorId-cross-runtime) — concrete messages earned when the compiler lands per §18.4 Growth policy. **Multi-runtime deferred** — v0.5.6 has one runtime per process, and `ActorId` cross-runtime comparison is reserved-only with an explicit "deferred to a future amendment" note. |
| Test framework (`@test` + `std::test` + `sploosh test`) is a first-class spec artifact, not a library add-on | The compiler needs a test harness to test itself; deferring testing to a third-party crate would force every implementer to invent the same surface differently. **Rust-shape assertions** (`assert_eq` / `assert_ne` / `assert_matches`) maximize Rust-trained-model recall and inherit the `Debug`-on-failure formatting users expect. **Per-test isolation actor** (§13.3.4) reuses §8 actor failure semantics — a panic in one test never aborts the runner, and there is no separate "test panic" mechanism to reason about. **`@test async fn`** runs on a fresh per-test runtime so async / actor / channel / select code is testable with no special syntax. **Property tests with `Gen<T>` + deterministic shrinking** (§13.3.6) ship in v0.5.5 rather than waiting for a later slice: shrinking is what makes property tests usable in CI, and locking the contract before the compiler emits its first `@property` keeps the trait shape stable. **Test-only prelude additions** (`assert_eq`, `assert_ne`, `assert_matches`, `TestFailure`, `Gen`, `Rng`) auto-import under `#[cfg(test)]` only — production code that references them is a compile error (`E1410` / `E1411`, reserved), preventing test code from leaking into release binaries. **`sploosh test` is deterministic with `--test-threads=1 --seed=<fixed>`** — byte-identical output across runs is the contract LLM agents and CI snapshot tests rely on. **No `@bench` in v0.5.5** — benchmarking has its own design space (warm-up, timer choice, statistical reporting) and is deferred. Doc tests are also deferred — they require a documentation-extraction pass the compiler does not have. |
| PROMPT-edition token budgets (`_CORE` ≤ 4,800 / `_WEB3` ≤ 1,500 cl100k_base) are CI-enforced ceilings calibrated to attention quality, portability, and per-token economics — not to frontier context-window capacity | Frontier context windows hit 1M+ tokens in 2026 and the combined PROMPT footprint sits at ~6,300 tokens (well under 1% of frontier capacity), so the budgets are deliberately **not** auto-scaled with context-window inflation. The constraint is three-fold: **(a) attention quality** — empirically, LLMs retrieve and reason worse from sprawling prompts even when they fit, so a tight reference is a more useful reference; **(b) prompt portability** across the long tail of smaller / on-device / 8K-context-window models that practitioners ship to edge environments and on-chain dev tooling, where any inflation here closes off real deployment surface; **(c) per-token economics** at ecosystem scale, where each PROMPT load is paid per developer session and per CI run across an entire community, so unbookkept growth has compounding cost. The `>` 100% / 90–100% / `<` 90% three-tier semantics (fail / warn / pass) catch genuine drift without flagging routine amendments, and the documented amendment path (raise the principle-7 number with explicit rationale) preserves the cost-signal of each ceiling bump — the v0.5.8 commit `bd26e8f` raising `_CORE` from `~4,000` to `~4,800` after the prompt split is the precedent. **Counterfactual considered and rejected**: auto-scale the ceiling with frontier context windows. Rejected because it sets a precedent of passive drift, makes growth unconscious rather than deliberate, and abandons every consumer that is not running on a frontier model — exactly the practitioners Sploosh targets at the edge of web3 deployment. Cross-references: §1 principle 7 (the budget numbers themselves), Appendix D v0.5.9 row, `scripts/check_prompt_budget.py` (the enforcer). |
| `Display` derivable, mirroring the `Debug`-derive shape | Manual `impl Display` is the most common boilerplate after `Debug` for any struct or enum that ends up in a log line, error message, or CLI output. Making it derivable removes that boilerplate for the common case while preserving manual override for types whose canonical rendering is not field-by-field (`Address`, `u256` units, anything with a domain-specific format). The shape mirrors `Debug` (`StructName { field: <field as Display>, ... }`) rather than introducing a new format-string DSL because (a) the format-string-on-the-derive path (e.g., `@derive(Display(format = "..."))`) is its own design space — string-template syntax, escape rules, runtime vs. compile-time validation — and locking it into v0.5.10 would foreclose that decision; (b) the field-by-field default is predictable, and predictability is the whole point of a derive (the LLM that writes `@derive(Display)` should be able to predict what comes out). The conflict rule (derive XOR manual impl) matches `Debug`'s. The recursive-Display field requirement matches Rust's behavior and surfaces missing impls at the derive site rather than at the call site. Cross-references: §3.10 standard traits table, §9.3 Display and Debug, §12.2 derive macros. |
| `ChainError` lives at `std::chain::ChainError` and is re-exported from the §13.1 prelude | Stdlib convention places an error type alongside the module that produces it — `chain::call` is the only intrinsic that returns `Result<T, ChainError>`, so `std::chain` is the natural home. The prelude re-export is an ergonomic exception, not a default: cross-contract calls are common enough on the on-chain target that requiring `use std::chain::ChainError;` for every fallible call signature would add friction without information value. The §11.4a definition stays canonical — the prelude entry and the `docs/stdlib/chain.md` page both reference §11.4a rather than duplicating the variant list, so a single edit point covers the type's shape. **Counterfactual considered and rejected**: leave `ChainError` un-prelude'd and require explicit `use std::chain::ChainError;`. Rejected because every on-chain function that calls another contract returns `Result<T, ChainError>` (or a wrapping error that contains it), and the import boilerplate would be unavoidable rather than opt-in. The prelude's existing on-chain-friendly types (`Address`, `u256`) set the precedent — `ChainError` slots in next to them. |
| `W0010` — `u256` off-chain arithmetic warning is **arithmetic-only** and **warn-by-default** | `u256` is a load-bearing on-chain primitive (Solidity-compatible storage slots, EVM word size, ABI-stable across the chain ecosystem) but a perf footgun off-chain: native and wasm have no 256-bit ALU, so every operator lowers to multi-instruction emulation (~10–50x slower than `u64`). The lint exists to surface this without forbidding the type, because legitimate off-chain uses exist — chain-bridge value plumbing, indexers replaying on-chain math, simulators of contract logic. **Arithmetic-only trigger** (not declarations / parameters / casts / literals) is the locked design: passing `u256` through off-chain code is free at runtime — only the *math* is expensive — and a declaration-firing lint would drown legitimate plumbing in noise that devs would learn to ignore wholesale. **Warn-by-default** (not allow-by-default) because the casual user — exactly the LLM-trained practitioner Sploosh targets — would never think to enable an off-by-default lint, and silent emulation is precisely the kind of correctness-preserving-but-perf-eroding trap that a deeply-trained `u256` muscle from Solidity carries into every off-chain context. The cost-signal needs to be loud at first sight and quiet on consenting demand (`#[allow(W0010)]` at site or module). Counterfactual considered and rejected: fire on declarations. Rejected because chain-bridge code mixing on/off-chain types is exactly the maintainer-aware case the lint should *not* punish; only *doing math* on the off-chain side is the footgun. Cross-references: §3.1 (`u256` type), §4.8 (integer overflow / arithmetic methods), §18.2 (warnings cluster), `docs/reference/compiler-errors.md` (registry row). |
| Pipe and method-chain forms for iterators are equivalent first-class syntaxes; no style is preferred | Both `vec.iter().map(f).collect()` and `vec.iter() \|> map(f) \|> collect()` lower to the same call sequence under §5.6's pipe rule (`expr \|> method(args)` ≡ `.method(args)`). The equivalence is not iterator-specific — it is the general pipe lowering applied to method calls — so collapsing to a single form would require either removing pipe-on-methods (loses `\|>` consistency with the rest of the language) or removing method-chain-on-`Iter` (loses Rust-trained-model recall and the established `.iter()` idiom). **Counterfactual considered and rejected**: pick one canonical form and lint the other. Rejected because the lowering is genuinely the same expression at the AST level after desugaring, so a stylistic prescription would be enforcing surface syntax for its own sake; LLMs and humans pick the form that reads better in context, and the spec's job is to document the equivalence rather than legislate aesthetics. The community is free to converge on conventions in code style guides over time. |

---

## 18. Compiler Diagnostics

The compiler's diagnostic surface is a first-class design artifact. Because
Sploosh is positioned as an AI-native language, machine-actionable errors —
stable codes, structured output, suggested fixes with applicability markers —
are the primary interface between the compiler and the LLM / IDE / human
reading the result. This section specifies the **format** of a diagnostic.
The stable **registry** of error codes lives in
`docs/reference/compiler-errors.md`. The compiler CLI surface for selecting
a rendering lives in `docs/tooling/build-system.md`.

### 18.1 Diagnostic record

Every diagnostic emitted by the compiler or runtime carries the following
canonical fields. Field names and semantics are stable across the renderings
described in §18.5.

| Field             | Type                | Meaning |
|-------------------|---------------------|---------|
| `code`            | `&str`              | `E<NNNN>` (error), `W<NNNN>` (warning), or `L<NNNN>` (lint). Stable identifier — see §18.4. |
| `severity`        | `Severity`          | One of `error`, `warning`, `help`, `note`. |
| `message`         | `String`            | One-line summary, sentence case, no trailing punctuation. |
| `primary_span`    | `Span`              | `{ file, byte_start, byte_end, line_start, line_end, col_start, col_end }`. Byte offsets are 0-based; line/column are 1-based. |
| `labels`          | `Vec<Label>`        | Each `{ span, message }`; the primary label's `span` equals `primary_span`. Additional labels attach supporting annotations at other spans. |
| `children`        | `Vec<Child>`        | Nested `{ severity, message, spans }` records, used to render `note:` and `help:` lines beneath the primary diagnostic. |
| `suggested_fixes` | `Vec<Fix>`          | Zero or more suggested edits — see §18.3. |
| `locale`          | `Option<String>`    | BCP-47 language tag for `message`, `labels[*].message`, and `children[*].message`. `None` ≡ English (`en`). The compiler at v1 always emits `None`. The field is **omitted from JSON output when `None`** (not serialized as `null`) — saves bytes; consumers default to `en` on absence. Reserved for future non-English diagnostic emission. |

Long-form explanations are addressable at `https://sploosh.dev/errors/{code}`
where `{code}` is the **lowercased** code (e.g. `e1101`). The diagnostic
record does **not** carry this URL — consumers construct it from `code`
deterministically. The hosted page is a future deliverable; the URL shape
is reserved at spec time so registry rows can be served from any static
host without record-format changes.

### 18.2 Error-code clusters

Codes are partitioned by topic so contributors know where to file new ones
and so readers can place a code at a glance. Ranges are reserved at spec
level; exceeding a cluster's range requires a spec amendment to declare a
new range, not silent reuse.

| Range         | Cluster | Topic |
|---------------|---------|-------|
| `E0001–E0999` | A       | Lexical / parser / basic syntax |
| `E1000–E1099` | B       | Type system, trait coherence, ownership, lifetimes |
| `E1100–E1199` | C       | On-chain (populated from v0.4.4 onward) |
| `E1200–E1299` | D       | Actors / concurrency |
| `E1300–E1399` | E       | FFI / extern |
| `E1400–E1499` | F       | Attributes / derives / directives |
| `W0001–W0999` | —       | Warnings |
| `L0001–L0999` | —       | Lints |
| `E9000+`      | —       | Internal compiler errors (ICE). Reserved; not user-facing under normal operation. |

### 18.3 Suggested-fix applicability

A `Fix` record carries `{ span, replacement, applicability, message }`. The
`applicability` field tells consumers (IDEs, LLM agents, formatters) whether
an automated tool may apply the edit without human review. The four values
are borrowed verbatim from rustc so Rust-trained models recognize them:

| Applicability       | Semantics |
|---------------------|-----------|
| `MachineApplicable` | Tools may auto-apply the fix. The replacement is complete and correct, and applying it preserves compilability (assuming no other errors). |
| `MaybeIncorrect`    | Rendered as a suggestion to a human; never auto-applied. |
| `HasPlaceholders`   | Contains placeholders (`<...>` or similar); a human must fill in before applying. |
| `Unspecified`       | No applicability declared. Consumers must treat as `MaybeIncorrect`. |

### 18.4 Stability contract

Once a code is published in `docs/reference/compiler-errors.md` at a
released version, its **semantic meaning is frozen**. Re-use of a number for
a different meaning is forbidden. Message text, label text, and
suggested-fix content may evolve freely between versions — only the
`code → meaning` mapping is stable.

Retirement path: a row may be marked `status: deprecated` with a
`superseded_by: <code>` pointer. The compiler continues to emit the
original code while documenting the replacement in the diagnostic's
`children` (as a `note:` line). The retired number is **not** reassigned.

### 18.5 Output formats

The compiler exposes three renderings of the §18.1 record; the CLI flag
that selects them is specified in `docs/tooling/build-system.md`:

- **`human`** (default). Rustc-style rendering: `error[E1101]: <message>`
  header line, `-->` source-span pointer, numbered gutter with the primary
  span highlighted and supporting labels attached, `note:` / `help:` child
  lines, and suggested-fix blocks with their applicability rendered
  inline.
- **`json`**. **Newline-delimited JSON** — one record per line, flushed
  immediately after each diagnostic so LLMs and IDEs can consume output
  mid-compile. Each record matches the §18.1 field layout. Every record
  carries a mandatory **`schema`** integer field as its **first** field,
  letting consumers negotiate before parsing the rest of the object. The
  initial value is `1`. Additive changes to the record (new optional
  fields) keep `schema: 1`; the value bumps to `2` only on a **breaking**
  schema change. The field is named `schema`, not `$schema` — Sploosh
  makes no JSON-Schema-conformance claim, so the leading `$` would be
  misleading. The field layout is stable across patch versions and
  additive across minor versions (new fields may be added; existing
  fields keep their names and types). Optional fields whose value is
  `None` are **omitted** from JSON output rather than serialized as
  `null` — `locale` is the canonical example.
- **`short`**. A single line per diagnostic:
  `<path>:<line>:<col>: <severity>[<code>]: <message>`. Optimized for
  `grep`-style log processing; omits labels, children, and suggested
  fixes.

Implementations may additionally expose a command that prints the
long-form explanation for a single code (conventionally
`sploosh --explain <code>`); see `docs/tooling/build-system.md`. The
long-form text is sourced from the local
`docs/reference/compiler-errors.md` registry, not from a network call.

### 18.6 LLM-integration contract

The `json` rendering is the primary artifact that agents parse. The
following invariants hold for every diagnostic in `json` mode. They are
what lets an LLM round-trip a fix-and-retry loop deterministically, and
they are what distinguishes Sploosh diagnostics from free-form compiler
output.

1. **Every diagnostic carries a `code`.** The compiler never emits a
   `"unknown"` placeholder. If no registered code applies, the compile is
   treated as an internal compiler error (`E9000+`) and the diagnostic
   reports the ICE code.
2. **Every `MachineApplicable` fix is complete.** When the compiler marks
   a fix with this applicability, its `replacement` string contains no
   placeholders and no `<...>` sentinels, and applying it unconditionally
   preserves compilability of the surrounding region assuming no other
   errors are present. This is a definitional property of the
   applicability level, not an aspiration — the compiler must not mark a
   fix `MachineApplicable` unless both conditions hold.
3. **`primary_span` is always populated.** File-less diagnostics (e.g.
   CLI argument errors) are reported under a synthetic `"<cli>"` file
   with byte offsets 0/0.
4. **`children` severities are one of `note` or `help`.** `error` and
   `warning` only appear at the top level — one top-level diagnostic per
   JSON record. Agents can parse records one at a time without tracking
   parent-child severity state.
5. **At most one `MachineApplicable` fix per diagnostic.** A diagnostic's
   `suggested_fixes` array contains at most one `Fix` with
   `applicability = MachineApplicable`. If the compiler can construct
   multiple complete-and-correct completions, it picks one or downgrades
   all of them to `MaybeIncorrect`. This lets agents in fix-and-retry
   loops auto-apply the single `MachineApplicable` fix without
   disambiguation logic.

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
| v0.4.3 | **Actor lifecycle states** (new §8.1a): explicit `INITIALIZING → READY → DEAD` state machine. `fn init(args) -> Self` is **infallible by signature and non-async** — writing `async fn init` is a compile error, and `init` panic transitions directly to `DEAD`. Messages sent to an INITIALIZING actor queue in the mailbox and are delivered once `init` returns; there is a happens-before edge from `init` completion to the first handler dispatch, so handlers never observe partially-constructed state. Handles returned by `spawn` may be immediately dead if `init` panics — first call observes `Err(ActorError::Dead)` or silent drop. **Message ownership rule rewritten by receiver type** (§8.2): `&mut self` methods (which may be invoked via `send`) must use owned parameters; `&self` methods (request/reply only — caller always blocks, stack always outlives the call) **may** take reference parameters. The rule now explicitly rejects `send handle.method()` on an `&self` method as a compile error. Resolves the previous §8.2/§8.3 contradiction on `Cache::get(&self, key: &K)`. Private (non-`pub`) actor methods retain their existing "references freely allowed" rule. **Handle drop semantics** (§8.2): dropping a `Handle<T>` — including the last live handle — has **no effect on actor lifetime**. Actors terminate only via runtime failure, supervisor termination, or runtime shutdown. Orphaned actors (no live handle, empty mailbox) are a known tradeoff of the non-reference-counted handle model; explicit self-termination is deferred. **Generic actor `Send` bounds** (§8.3): all type parameters on an `actor` declaration must carry `Send`, not only those used in `pub` method signatures. **Supervisor restart semantics** (new §8.7a): restart always runs a fresh `init` with the supervisor's stored construction arguments — no state preservation across restart; old `Handle<T>` values become permanently dead and are not transparently redirected to the restarted instance (callers re-fetch from the supervisor); queued mailbox messages are discarded; init failures count toward `max_restarts`; `rest_for_one` requires children in an ordered collection, with a compile-time warning and `one_for_one` fallback if dynamic. **Supervisor restart window is sliding** (§8.7): `window_secs` is a wall-clock sliding window over each restart's timestamp, not a fixed window with reset boundaries. **Mailbox-full + actor-dies race** (§8.11): blocked senders wake immediately on destination death; `send` drops silently, `send_timeout` returns new variant `SendError::Dead` (added to §8.5 `SendError`), request/reply returns `Err(ActorError::Dead)`. Wake order unspecified. Supervisor restart does not redirect blocked senders. **Re-entrant call detection** (new §8.10.1): direct self-calls (A → A via request/reply on own handle) return new variant `ActorError::SelfCall` immediately via an O(1) runtime check — variant added to the `ActorError` enum in §8.8. Multi-actor cycles (A → B → A) are documented as a hazard but not detected in v0.4.3. Fire-and-forget self-sends (`send self.handle.method(args)`) are legal and are the correct self-scheduling pattern. Cross-references §11.3 on-chain reentrancy as a distinct mechanism. **Blocking operations in handlers** (new §8.11a): actor handlers run in an async context; the stdlib is already async-only, so no new forbid is needed there. **FFI:** calling a synchronous `extern "C"` function from inside a handler (directly or transitively) is a compile error. Handler-safe FFI must be declared `extern "C" async` (§4.9) — the compiler emits an awaitable wrapper that offloads to the runtime's blocking pool. `spawn_blocking` intrinsic deferred. **Select arm fairness** (§8.6): arms are checked top-to-bottom deterministically (not round-robin), making `select` reproducible under test. **Actors forbidden in `onchain`** (§8.1, §11.1, §12.3): the `actor` keyword, `spawn`, `send`, `send_timeout`, `select`, `timeout(ms)`, `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>`, `@supervisor`, `@mailbox`, the `async` function modifier with its `.await` operator, and `extern "C"`/`extern "C" async` FFI are all compile errors inside `onchain` modules. Transitive imports of actor-using native modules through pure-function boundaries remain allowed — the forbid is on *spawning inside onchain*, not on depending on actor-using code. **Deferred to a future amendment:** `init() -> Result<Self, E>`, async `init`, explicit `handle.stop()` / `stop c` intrinsic, `spawn_blocking` intrinsic, multi-actor cycle detection, `ChildSpec<T>` as a language-visible type. No grammar changes. No new keywords (still 40). |
| v0.4.4 | **On-chain semantics — Cluster C** (§11): closes the four design-heavy gaps that prevented EVM/SVM codegen from starting. **Storage layout** (new §11.1a): target-pluggable abstraction with Solidity-compatible EVM reference realization — sequential `u256` slots in declaration order, Solidity-rule packing within slots, `Map<K, V>` entries at `keccak256(abi.encode(key, map_slot))`, nested maps recurse, `Vec<T>` / `String` with length at slot and data at `keccak256(slot)`, `[T; N]` inline. SVM layout deferred to a future Solana amendment; Sploosh surface stays identical across targets. **Reentrancy guard mechanism** (new §11.3a): runtime per-contract boolean flag set on entry to any non-`@reentrant` `pub` on-chain function and cleared on return (success, error, or revert). Cross-contract re-entry into a guarded function reverts with new error `ChainError::Reentrancy`; `@reentrant` disables the check and the set for that function only. Gas cost is qualitative (one TLOAD + one TSTORE per guarded call on EIP-1153 EVM forks (Cancun+), SLOAD/SSTORE fallback on earlier forks). Explicitly distinguished from §8.10.1 actor `SelfCall` — same word, different layers. **Cross-contract ABI and call semantics** (new §11.4a): new surface syntax `extern onchain mod X { pub fn ...; ... }` declares callee signatures at compile time; `chain::call(addr, callee, args) -> Result<T, ChainError>` blocks synchronously on EVM (lowers to `CALL`), Solidity ABI is the reference argument encoding on EVM, `?` propagates `ChainError::Reverted { data: Vec<u8> }` with revert bytes bounded by `RETURNDATACOPY` semantics. New error enum `ChainError { Reverted, OutOfGas, Reentrancy, InvalidTarget, DecodingError }` added to the on-chain error surface. No delegatecall in v0.4.x (deferred to v0.5.0). SVM divergence via Solana CPI with preserved user-level surface; concrete ABI deferred. **Explicit contrast with `extern "C"` (§4.9)**: both nest under `extern`, but calling conventions, safety models, and error surfaces differ — not interchangeable. **Gas model** (new §11.7a): target-pluggable metering abstraction. EVM references Yellow Paper + active-hard-fork EIP cost tables (Sploosh does not redefine opcode costs); `ctx::gas_remaining() -> u256` EVM-only, `#[gas_limit(N)]` EVM-only advisory in deployed ABI metadata. SVM uses compute units; `ctx::compute_units_remaining() -> u64` SVM-only. All three are compile errors on native and wasm. **Out-of-gas semantics**: transaction-wide revert, all storage mutations and emitted events unwound, and revert is **unaffected by per-function attributes including `@reentrant`** (explicit invariant). Transient-state unwind clears the reentrancy flag on revert, so failed calls cannot leave a contract with its guard stuck set. **`#[indexed]` event field marker** (§11.5, §12.3): up to three indexed fields per event variant on EVM (topic slots 1–3; topic 0 is the signature hash); compile error on more. SVM accepts `#[indexed]` for source-compatibility but treats it as a no-op. **§13.0 intrinsics table**: `ctx::gas_remaining` context column tightened to EVM-only, new row for `ctx::compute_units_remaining` (SVM-only), `chain::call` signature updated to `Result<T, ChainError>`, `storage::*` rows reference §11.1a, `chain::call` row references §11.4a. **§16 grammar**: `extern_block` production extended — `extern_target = STRING_LIT | "onchain" "mod" IDENT` — and `extern_fn` allows optional `pub`. No new keywords (still 40). No new item kinds; `extern onchain mod` is an extern-block variant. **Deferred to v0.5.0**: cross-contract ABI emission artifacts (bytecode + ABI JSON + metadata file), WASM target variants (`wasm32-unknown-unknown` vs `wasm32-wasi`), delegatecall support, SVM storage layout details, SVM CPI concrete ABI, per-call gas forwarding annotation. |
| v0.5.0 | **Removed the `none` keyword** (§2.3, §16). Per the independent PR #9 review, `none` was reserved in the §2.3 keyword list and appeared as a literal in the §16 grammar `literal` production, but every example, every guide, and the `docs/` tree generally used `None` (the `Option::None` constructor exported from the §13.1 prelude). Lowercase `none` was reserved in two definitional sites and used in zero practical sites — the keyword reservation served no purpose while creating a contradiction with the prelude. Removed from `docs/spec-plans/LANGUAGE_SPEC.md` §2.3 and §16, and from the `docs/reference/keywords.md` and `docs/reference/grammar.md` mirrors. Keyword count: 40→39 (losing `none`). The capitalized `None` — an identifier resolving to `Option::None` via the prelude — is unchanged and remains the sole form for an absent `Option` value. No grammar reshape beyond the deleted alternative; no other sections touched. This amendment opens the v0.5.x cycle with a mechanical correctness fix identified by the PR #9 review (severity Blocker, action L1). |
| v0.5.1 | **Compiler Diagnostics specification** (new §18). Formalizes the compiler's diagnostic contract as a first-class spec artifact — the highest-leverage missing piece for the AI-native positioning. New §18.1 Diagnostic record defines the canonical field layout (`code`, `severity`, `message`, `primary_span`, `labels`, `children`, `suggested_fixes`, `explanation_url`) that all renderings must preserve. §18.2 Error-code clusters reserves ranges: `E0001–E0999` lexical (A), `E1000–E1099` type/trait/ownership (B), `E1100–E1199` on-chain (C, already in use), `E1200–E1299` actors/concurrency (D), `E1300–E1399` FFI (E), `E1400–E1499` attributes (F), `W0001–W0999` warnings, `L0001–L0999` lints, `E9000+` ICE. §18.3 Suggested-fix applicability adopts rustc's vocabulary verbatim (`MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, `Unspecified`) so Rust-trained models recognize the levels. §18.4 Stability contract: code→meaning is frozen on release; retired codes are marked `status: deprecated` with a `superseded_by` pointer and are never reassigned. §18.5 Output formats: `human` (default, rustc-style), `json` (newline-delimited JSON, one record per line, stable field layout with optional `$schema`), `short` (single line per diagnostic, grep-friendly). §18.6 LLM-integration contract: four invariants that hold for every diagnostic in `json` mode — every diagnostic carries a code; `MachineApplicable` fixes are complete (applying them preserves compilability); `primary_span` is always populated (file-less diagnostics use a synthetic `"<cli>"` file); `children` severities are limited to `note` / `help`. Explicit non-commitments: the spec does **not** mandate a hosted URL for `explanation_url` (implementations may leave it `None`) and does **not** commit to a specific JSON Schema artifact for `$schema` (draft-7 emission is a future follow-up). New §17 Design Decisions row documents the format-as-AI-native-lever rationale. **Registry expansion**: `docs/reference/compiler-errors.md` rewritten to distinguish "format" (§18) from "registry" (this file), adds `Cluster` and `Status` columns to the existing E1101–E1109 on-chain rows, and reserves cluster-header placeholders for the A/B/D/E/F/W/L/ICE ranges with TODO entries. Adds a "Growth policy" block (4 rules: registry-first workflow, spec-section anchoring, frozen-on-publish, deprecate-don't-reassign). **Tooling**: `docs/tooling/build-system.md` gains a Compiler Flags subsection documenting `--error-format=<human\|json\|short>` (default `human`) and `--explain <code>` (prints long-form explanation sourced from the local registry, not a network call). No new keywords (39 unchanged). No grammar changes. Closes PR #9 review Blocker U1. **Principle #7 softened** (§1): the 4,000-token claim is now framed as a soft target rather than a hard budget, acknowledging that the PROMPT edition was already 4,077 tokens (cl100k_base) before v0.5.1 and the Diagnostics bullet added ~133 more. This partially addresses review action L8 by tightening the claim to match reality; the stricter CI-enforcement path remains a strategy decision for a future amendment. |
| v0.5.10 | **Cleanup batch — six small-item slice contributions** (closes issue #20, slice 8 of 8 in the v0.5.3–v0.5.10 sequence). Stacked-PR architecture: a `spec/v0.5.10-cleanup-base` integration branch carried six sub-PRs (#34, #35, #36, #37, #38, #39) — all merged via squash — followed by this single integration commit consolidating the line 1 header bump, the footer bump, the Appendix D row, and a §17 thematic reorder of the v0.5.10 rows. Sub-PRs deliberately deferred those cross-cutting edits to avoid merge conflicts at the base-branch level. **(1) `Display` derivable, mirroring `Debug`** (PR #37): manual `impl Display` is the most common boilerplate after `Debug` for any struct or enum that ends up in a log line, error message, or CLI output, and making it derivable removes that boilerplate without foreclosing the format-string-on-the-derive path (a separate design space deferred). The shape mirrors `Debug` — `StructName { field: <field as Display>, ... }` — so the derive output is predictable; the conflict rule (derive XOR manual impl) matches `Debug`'s; the recursive-Display field requirement surfaces missing impls at the derive site. Cross-references: §3.10 standard traits, §9.3 Display and Debug, §12.2 derive macros. **(2) `ChainError` lives at `std::chain::ChainError` and is re-exported from the §13.1 prelude** (PR #38): stdlib convention places an error type alongside the module that produces it — `chain::call` is the only intrinsic that returns `Result<T, ChainError>`, so `std::chain` is the natural home. The §11.4a definition stays canonical; the prelude entry and `docs/stdlib/chain.md` reference §11.4a rather than duplicating the variant list. The prelude re-export is an ergonomic exception, not a default — every on-chain function that calls another contract returns `Result<T, ChainError>`, so requiring an explicit `use std::chain::ChainError;` would add unavoidable boilerplate. **(3) E11xx Cluster C registry audit** (PR #35 + cubic-fix): the §18 cluster C range (`E1100–E1199`) had reserved-only entries; the audit fills concrete rows for `E1110`–`E1123` covering on-chain prohibitions (actor primitives, async, FFI, float math, `@fast_math`, `@overflow(wrapping)`, `Shared<T>`, `std::test`, `std::actor::observe`, `ActorId`, host stdlib modules). Slot `E1114` is intentionally vacant per §18.4 (frozen-on-publish), reserving it for a future on-chain prohibition without disrupting the existing numbering. **(4) `W0010` — `u256` off-chain arithmetic warning** (PR #39 + cubic-fix): `u256` is a load-bearing on-chain primitive but a perf footgun off-chain (no 256-bit ALU on native or wasm; ~10–50x slower emulation). The new lint is **arithmetic-only** (declarations / parameters / casts / literals do not fire — chain-bridge plumbing stays quiet) and **warn-by-default** (silent emulation is exactly the trap that a Solidity-trained `u256` muscle carries off-chain). The cubic-fix iteration canonicalized the trigger list once between the §3.1 prose, §4.8 cross-reference, and the `docs/reference/compiler-errors.md` registry row so the three sites agree by construction. Suppression at site or module via `#[allow(W0010)]`. **(5) Pipe and method-chain iterator forms are equivalent first-class syntaxes** (PR #34): a §7 note clarifying that `vec.iter().map(f).collect()` and `vec.iter() |> map(f) |> collect()` lower to the same call sequence under §5.6's pipe rule. Neither form is preferred — the equivalence is not iterator-specific (it is the general pipe lowering applied to method calls) — and the spec's job is to document it rather than legislate aesthetics. **(6) Infra-batch** (PR #36): Dependabot `pip` ecosystem enabled (`scripts/requirements.txt`, weekly), Dependabot commit-prefix changed to empty string (matches the AGENTS.md no-`feat:`/`fix:` rule for human authors and bot commits alike), `.github/pull_request_template.md` adds a "Spec-only PR" checkbox so spec PRs can mark Build Targets Tested as N/A without leaving the section blank, and `docs/rationale/why-sploosh-looks-this-way.md` audited and updated for the v0.5.9 principle 7 framing (attention quality + portability + per-token economics, not frontier-context capacity). **§17 Design Decisions Log**: the four sub-PRs that added §17 rows (#34, #37, #38, #39) are reordered into a thematic block — Display derivable → `ChainError` module home → `W0010` u256 → iter-pipe equivalence — with the v0.5.9 PROMPT-edition row (which was added before v0.5.10's contributions) anchored at the top of the v0.5.10 thematic block. PR #35 (E11xx audit) and PR #36 (infra-batch) intentionally added no §17 row — the audit fills reserved slots per §18.4 Growth policy without locking new design rationale, and the infra-batch is operational hygiene rather than language design. **PROMPT-edition mirrors**: `_CORE` (Display derive line in `## Standard Traits`) and `_WEB3` (`ChainError` notes in `## Cross-contract Calls (§11.4a)`) bumped to (v0.5.10) headers. CI-enforced PROMPT-budget (per §1 principle 7 / v0.5.9 enforcer) post-integration: `_CORE` 4,717 / 4,800 (98.3%, warn band, headroom 83 tokens); `_WEB3` within budget. **Mirror-doc sweep**: doc-level current-version markers bumped from `v0.5.9-draft` to `v0.5.10-draft` in `VISION.md:119`, root `AGENTS.md:9`, `.factory/droids/sploosh-spec-steward.md:9`, plus the spec line 1 + footer. Historical narrative references to v0.5.9 (e.g., "as of v0.5.9 the ceilings are CI-enforced", `scripts/check_prompt_budget.py:11`, `docs/rationale/why-sploosh-looks-this-way.md:54`) are intentionally preserved — those describe when each contract landed, not the current spec version. **Slice plan complete**: v0.5.10 closes the v0.5.3–v0.5.10 cleanup sequence (8 of 8 done). The next milestone is the **compiler bootstrap** — lexer / parser targeting §16 EBNF. No new keywords (39 unchanged). No grammar changes beyond the Display-derive entry already documented in §12.2. **Operational note**: this slice exercised a true stacked-PR / sub-PR pattern for the first time in the project's history. The git-worktree strategy validated as a true-parallel-worker mechanism; an in-process Python script proved useful as a fallback for atomicizing git operations in shared-workspace scenarios where the worker can't take its own checkout. **Deferred**: hosted explanation page at `https://sploosh.dev/errors/{code}` (still v1.0); `schema: 2` JSON record bump (still reserved for the first breaking §18.1 change); format-string-on-the-`Display`-derive (its own design space). |
| v0.5.9 | **PROMPT token-budget CI enforcement** (§1 principle 7 rewrite + new §17 row + new `scripts/check_prompt_budget.py` + new `.github/workflows/prompt-budget.yml`). Closes issue #19 (slice 6 of 8 in the v0.5.3–v0.5.10 sequence). The v0.5.1 softening framed the PROMPT budget as a soft target with no enforcement; the v0.5.8 split landed at 4,616 (`_CORE`) / 1,327 (`_WEB3`) `cl100k_base` tokens against ~4,800 / ~1,500 ceilings, and the missing piece was preventing silent drift across future amendments. **CI workflow**: `prompt-budget.yml` runs on every `pull_request` against `main` and on `push` to `main` (no path filter — runs on every PR regardless of which files changed), checks out the repo, sets up Python 3.12 with pip caching keyed on `scripts/requirements.txt`, installs `tiktoken==0.12.0`, and invokes `scripts/check_prompt_budget.py`. Concurrency group `prompt-budget-${{ github.ref }}` cancels in-flight runs on push. **Three-tier semantics**: `< 90%` of ceiling → silent pass (no PR clutter); `90–100%` → warn (printed line `WARN: _CORE at 96.2% of budget (4616/4800)` so the next contributor sees they're close, exit 0); `> 100%` → fail (exit 1, blocks merge until trim or explicit principle-7 budget-bump PR). Measured at PR time: `_CORE` 4,616/4,800 = 96.2% (warn band); `_WEB3` 1,327/1,500 = 88.5% (silent pass). **Script shape** (`scripts/check_prompt_budget.py`): ~75 lines, no dependencies beyond `tiktoken` + stdlib. Parameterizable via `--core-ceiling` / `--web3-ceiling` / `--warn-at` flags so the budgets aren't magic numbers; defaults match the principle-7 numbers (`4800` / `1500` / `0.9`). Runnable standalone from repo root. **§1 principle 7 rewritten**: replaces the v0.5.1-era "fits in a system prompt" / "soft target" framing with an AI-native density rationale that acknowledges the 1M+ context-window era. The budgets are not constrained by frontier capacity (combined ~6,300 tokens is well under 1% of a 1M-context window) — they are constrained by **(a) attention quality** (LLMs retrieve worse from sprawling prompts even when they fit), **(b) prompt portability** across the long tail of smaller / on-device / 8K-context-window models, and **(c) per-token economics** at ecosystem scale where each prompt is loaded N times across a developer ecosystem. The v0.5.1 "soft target, no enforcement" hedge becomes "soft target, CI-enforced ceiling, with documented amendment path" — a strengthening of the contract, not a loosening. The amendment path is documented explicitly: when the soft ceiling is genuinely too tight, the right move is an explicit principle-7 amendment that bumps the number with rationale, and the v0.5.8 commit `bd26e8f` (raising `_CORE` from `~4,000` to `~4,800` after the prompt split) is cited as precedent. **§17 Design Decisions Log row added** — captures the "context-window-aware budget rationale" choice that future readers in 2028+ would otherwise wonder about: why didn't the 2026 spec just track frontier context? Answer rooted in attention quality, portability, and economics, with the auto-scale-with-frontier-context-windows counterfactual considered and explicitly rejected. **Mirror-doc sweep**: `docs/spec-plans/AGENTS.md` Files-table entries for `_CORE` / `_WEB3` updated to note CI-enforced ceilings; root `AGENTS.md` references the principle in "Documentation = Language"; `.factory/droids/sploosh-spec-steward.md` references updated. **Ride-along**: stale `v0.5.2-draft` doc-level version markers in `VISION.md:119` and `AGENTS.md:9` (root) bumped to `v0.5.9-draft` (folded into a separate commit on the same PR; surfaced as out-of-scope drift by the v0.5.8 cubic-fix worker, six versions stale, bumping once to current is correct). No new keywords (39 unchanged). No grammar changes. No new diagnostic registry entries. PROMPT-edition mirrors (`_CORE` / `_WEB3`) unchanged in content — only the enforcer is new. |
| v0.5.8 | **Prompt-edition split** (§1 principle 7 amendment). Closes issue #18 (PROMPT split / split slot 5 of 8 in the v0.5.3–v0.5.10 sequence). The combined `LANGUAGE_SPEC_PROMPT.md` (~4,277 cl100k_base tokens at v0.5.7) is replaced by two artifacts: `LANGUAGE_SPEC_PROMPT_CORE.md` carries the language core (syntax, types, ownership, math, actors, observability, errors, iterators, modules, runtime, async, FFI, attributes, testing, diagnostics, manifest) plus a new "Common LLM Mistakes" appendix; `LANGUAGE_SPEC_PROMPT_WEB3.md` carries the §11 on-chain surface (`onchain mod`, `storage`, `ctx`, storage layout §11.1a, reentrancy guard §11.3a, cross-contract calls §11.4a, gas/CU §11.7a, events §11.5, `@payable` / `@reentrant`, `ChainError`). The on-chain prohibition list is **deliberately duplicated** in both files (~150 tokens overhead) so each audience sees the compact reminder without loading the other artifact. **§1 principle 7 updated**: per-file soft targets are roughly 4,800 tokens (`_CORE`) and roughly 1,500 tokens (`_WEB3`), framed as soft targets per the v0.5.1 softening; the strict CI-enforcement path is the v0.5.9 follow-up. The `_CORE` target was calibrated upward from an initial `~4,000` to `~4,800` after the split landed at 4,616 measured tokens, preserving the v0.5.1 soft-target framing rather than re-trimming high-value reference content (notably the §4.10 float-method enumeration); `_WEB3` stayed at `~1,500` (measured 1,327, ~173-token headroom). **Retired combined file**: `LANGUAGE_SPEC_PROMPT.md` is kept in place (so inbound links don't 404) with a short redirect note pointing at the two new files. **New "Common LLM Mistakes" appendix** (last section of `_CORE`) — 10 single-line restatements of existing rules covering lifetime annotations, `as`-numeric-only, pattern-binding move semantics, capitalized variants, actor `&mut self` vs `&self` parameter rules, checked arithmetic, `Shared<T>` immutability, pipe + `?` precedence, test-assertion borrow semantics, and `chain::call` ergonomics. No new spec semantics — purely a denser presentation of existing rules. **Mirror-doc sweep**: `docs/spec-plans/AGENTS.md` Files table updated to list both new files with per-file budgets; root `AGENTS.md` "Documentation = Language" mirror table and JIT Index updated; `VISION.md` reference list updated; `.factory/droids/sploosh-spec-steward.md` mirror reference updated; `docs/spec-plans/LANGUAGE_SPEC_REVIEW.md` historical numbers preserved. No grammar changes. No new keywords (39 unchanged). No new diagnostic registry entries. **Slice plan**: GitHub issue titles for #19/#20 not renumbered — the slice number labels (v0.5.9 / v0.5.10) are the maintainer-locked plan slot labels. |
| v0.5.7 | **Compiler diagnostic format punch-list** (§18.1, §18.5, §18.6 amendments). Closes issue #26 (R3 Blocker from `LANGUAGE_SPEC_REVIEW.md` §4.5/§7) — fine-grained tightening of the §18 format spec landed in v0.5.1, with four maintainer-locked decisions oriented toward the AI-native consumer. **(1) `explanation_url` field removed from §18.1.** The previous wording — "implementations may always leave this `None`" — was a non-commitment that added JSON bytes for no information. Replaced with a canonical URL template `https://sploosh.dev/errors/{code}` where `{code}` is the **lowercased** code (e.g. `e1101`); consumers construct the URL from `code` deterministically, no per-record bytes, no `None`-handling branch. The hosted page is a future deliverable; the URL shape is reserved at spec time so registry rows can be served from any static host without record-format changes. **(2) `schema: 1` mandatory per-line integer field in `json` mode.** Renamed from `$schema` to `schema` to drop the JSON-Schema-conformance connotation Sploosh has not claimed. **First field** in every JSON object so consumers can negotiate before parsing the rest. Bumped to `2` only on a **breaking** schema change; additive changes (new optional fields) keep `schema: 1`. Replaces the previous "optional, implementation-defined, may omit entirely" wording — the field is now mandatory on every record (every line, not header-line). **(3) `locale: Option<String>` reserved in §18.1 record.** BCP-47 language tag, `None` ≡ English (`en`), v1 always emits `None`, field is **omitted from JSON output when `None`** rather than serialized as `null` — saves bytes and consumers default to `en` on absence. Reserves the slot today so non-English diagnostic emission later is additive (no `schema: 2` bump). The omit-when-`None` rule is restated in §18.5 as a generic field convention applicable to all future optional fields, with `locale` as the canonical example. **(4) At-most-one `MachineApplicable` fix per diagnostic** (new §18.6 invariant 5). A diagnostic's `suggested_fixes` array contains at most one `Fix` with `applicability = MachineApplicable`; if the compiler can construct multiple complete-and-correct completions it picks one or downgrades all to `MaybeIncorrect`. Lets fix-and-retry agents auto-apply the single `MachineApplicable` fix without disambiguation logic. The other four §18.6 invariants are unchanged. **Mirror-doc sweep**: `LANGUAGE_SPEC_PROMPT.md` Diagnostics paragraph updated for all four decisions; `docs/reference/compiler-errors.md` cross-references verified (no content edits — the registry never carried `explanation_url`/`$schema`/`locale`); `docs/tooling/build-system.md` `--error-format=json` and `--explain` prose verified consistent. No new keywords (39 unchanged). No grammar changes. No new diagnostic registry entries. **Slice renumbering** (the v0.5.3–v0.5.9 plan slot labels): the previously-planned v0.5.7 (PROMPT split / #18) → v0.5.8; v0.5.8 (token-budget CI / #19) → v0.5.9; v0.5.9 (cleanup batch / #20) → v0.5.10. GitHub issue titles for #18/#19/#20 are not renumbered — that is the maintainer's call. |
| v0.5.6 | **Actor observability — `std::actor::observe`, handle introspection, supervisor restart history** (new §8.12, with rippling additions to §13.0 intrinsics, §13.1 prelude, §13.2 core modules, §12.1 attributes, §17 design log). Closes issue #17 (slice 4 of 7 in the v0.5.3–v0.5.9 sequence). The runtime needs an answer to *what is this actor doing right now* before the compiler exists; speccing the observability surface before codegen prevents every implementer from inventing the same shape differently. **Hybrid placement** (§8.12.1, §8.12.2): cheap, constant-time reads live as direct methods on `Handle<T>` — `mailbox_len()`, `mailbox_capacity()`, `alive()`, `actor_id()` — all `&self`, infallible, available on dead handles. Richer queries (`actor_info(handle)`, `actors()`, `actors().by_supervisor(sup)`, `actors().by_name(name)`) live in the new `std::actor::observe` module. **Supervisor-rooted restart history** (§8.12.3): three new methods on `Handle<S>` when `S` is `@supervisor`-decorated — `restart_count(child) -> Result<u32, ObserveError>`, `restart_history(child) -> Result<Vec<RestartEvent>, ObserveError>`, `children() -> Iter<ActorInfo>`. New `RestartEvent { timestamp_ms_since_spawn, cause }` and `ObserveError::NotASupervisedChild`. Non-supervised actors have no restart path (§8.7) and therefore no history to expose. **Dead-actor snapshot retention** (§8.12.4): on transition to `DEAD` the runtime captures an `ActorInfo` with `lifecycle_state = Dead` and a populated `death_cause` — `RuntimeFailure { panic }`, `Stopped`, `Killed`, `Supervised { restart_pending }`, or `RuntimeShutdown`. Retained as long as any `Handle<T>` clone targeting the actor remains live (refcount-driven retention on the snapshot side-table — the **only** refcount in the actor model, explicitly distinct from the actor's own non-refcount lifetime in §8.2). **`ActorId` type** (§8.12.5): opaque `Copy + Eq + Hash`, monotonically assigned at spawn, never reused even after death. Not `Send` across runtime instances; multi-runtime story deferred. **`ActorInfo` snapshot record** (§8.12.2): `id`, `name` (unqualified actor type name), `spawn_location` (best-effort file:line), `supervisor: Option<ActorId>`, `lifecycle_state` (new `LifecycleState { Initializing, Ready, Draining, Dead }` enum), `mailbox_len`, `mailbox_capacity`, `death_cause: Option<DeathCause>`. **Cost model** (§8.12.6): per-actor ~24 bytes registry + atomic mailbox counter (already paid for backpressure); per-supervised-child ring buffer of last *N* `RestartEvent`s (default 16, ~384 bytes); per-snapshot ~256 bytes until last handle drop. `observe::actors()` is O(N_actors), not for hot paths. **Always-on in every build mode** — no `@observable`, no debug-only gating, no feature flag. Maintainer-locked tradeoff: pay the bookkeeping bytes always rather than letting users discover unobservability in production. **`@supervisor` extended with optional `restart_history: N` parameter** (§12.1), default 16. No new attribute, no new keyword. **§13.0 intrinsics**: four new rows for `Handle<A>.mailbox_len/mailbox_capacity/alive/actor_id` (signature `fn(&Handle<A>) -> usize|usize|bool|ActorId`, all `not onchain`). `std::actor::observe::*` items are stdlib, not intrinsics. **§13.1 prelude** adds `ActorId` after `Handle, JoinHandle`. **§13.2 core modules** adds a new `std::actor` row (native, wasm — actor observability and introspection). **On-chain prohibition** (§8.12.7): every `Handle<T>` method, the `std::actor::observe` module, and `ActorId` are compile errors inside `onchain` modules — restated from the existing §11.1 / §12.3 actor prohibition; `Handle<T>` itself is already an on-chain compile error so all its methods are too. **Stdlib mirror**: new `docs/stdlib/actor.md` page with Targets table, full Sploosh signatures, `observe` API, supervisor-rooted methods, and the `ActorId` / `ActorInfo` / `DeathCause` / `RestartEvent` / `LifecycleState` / `ObserveError` type reference. **Guide mirror**: `docs/guide/actors-and-concurrency.md` adds an "Observability" section after "Stopping Actors" with worked examples for tailing mailbox depth, walking a supervision tree, and triaging a dead actor's death cause. **Runbook mirror**: `docs/runbooks/actor-debugging.md` rewritten to be the operational counterpart to §8.12 — recipes for "is this actor stuck", "why did this actor die", "which actors are pinning memory", "what's our supervisor tree", with a troubleshooting table at the bottom. **Reference mirrors**: `docs/reference/attributes.md` extends the `@supervisor` row with the `restart_history` parameter; `docs/reference/compiler-errors.md` reserves two new Cluster D slots — `E1210` (observability method called on a non-supervised parent) and `E1211` (`ActorId` comparison across runtime instances; multi-runtime deferred). **PROMPT edition**: a one-paragraph "Observability" block added in the Concurrency section so LLMs know the API exists. **§17 Design Decisions Log** adds a new v0.5.6 row capturing the five maintainer-locked choices (hybrid placement, supervisor-rooted history, global runtime registry, snapshot-until-last-handle-drop, always-on bookkeeping). No grammar changes. No new keywords (39 unchanged). |
| v0.5.5 | **`std::test` framework — first-class spec artifact** (new §13.3, with rippling additions to §13.0 intrinsics, §13.1 prelude, §12.1 attributes, §17 design log). Closes issue #16 (slice 3 of 7 in the v0.5.3–v0.5.9 sequence) and review action U3. The compiler needs a test harness to test itself; speccing the surface before the compiler lands prevents every implementer from inventing the same shape differently. **Three new test-only intrinsics** (§13.0): `assert_eq(a, b)` and `assert_ne(a, b)` (Rust-shape, `T: Eq + Debug`, borrow operands so non-`Copy` values aren't consumed, report both sides via `Debug` on failure) and `assert_matches(value, pattern)` (special form using §5.2 match patterns; pattern bindings are not available after the assertion). All three are compile errors outside `@test`-annotated functions or `#[cfg(test)]` modules — diagnostic `E1410` reserved. **Test discovery and layout** (§13.3.2): unit tests live inline in `#[cfg(test)] mod tests { ... }`; integration tests live as standalone crates under `tests/*.sp` and only see the package's `pub` surface. Doc tests deferred. **Failure semantics — per-test isolation actor** (§13.3.4): each test runs inside its own runtime-spawned actor with a one-shot completion channel. Three observable outcomes: `Ok(())` (pass), `Err(TestFailure)` (returned-Err shape; `?`-friendly with the new `From<E>` for `E: Error` blanket on `TestFailure`), and actor death (panic, observed as `Err(ActorError::Dead { panic: Some(msg) })`). Reuses §8 actor failure semantics so there is no separate "test panic" mechanism to reason about — a single failing test never aborts the runner. **`async @test fn ...` is permitted** (§13.3.5): the runner spawns a fresh runtime per test; `.await`, channels, `select`, timeouts, and user-spawned actors all work as in production code. Tests that own actors clean them up via §8.2a `handle.stop()` / `.kill()` or rely on the runtime-shutdown sweep. **Property tests with `@property` attribute** (§13.3.6, new sibling to `@test` in §12.1): runner generates 256 cases per property by default, shrinks failures to a minimum reproducer, and reports both original and shrunk inputs with the RNG seed for reproduction. New `Gen<T>` trait (`type Item; fn generate(rng, size); fn shrink(value) -> Iter<Item>`) with prelude impls for every primitive integer, `bool`, `f32`/`f64`, `char`, `String`, `Vec<T: Gen>`, `Option<T: Gen>`, `Result<T: Gen, E: Gen>`, and tuples up to arity 12. Deterministic shrinking required — same seed reproduces the same shrunk minimum byte-for-byte. **`sploosh test` runner contract** (§13.3.7): flags `--filter`, `--exact`, `--test-threads`, `--nocapture`, `--seed`, `--cases`, `--format`. Deterministic output under `--test-threads=1 --seed=<fixed>`. Exit codes `0` (pass), `1` (any test failed), `2` (runner error). **`std::test` public surface** (§13.3.8): `TestFailure`, `Gen`, `Rng` library types; `assert_eq` / `assert_ne` / `assert_matches` re-exported for documentation locality. All `#[cfg(test)]`-only — outside-test references are `E1411` (reserved). **Test-only prelude additions** (§13.1): `assert_eq`, `assert_ne`, `assert_matches`, `TestFailure`, `Gen`, `Rng` auto-import only under `#[cfg(test)]`, preventing test code from leaking into release binaries. **`std::test` is a compile error inside `onchain` modules** — on-chain code is tested off-chain by spawning a simulated execution context; the `@onchain_test` shape is deferred to a future amendment. **Tooling mirrors**: `docs/stdlib/test.md` rewritten from 8-line stub to full API reference; `docs/tooling/build-system.md` adds the test-runner flag table; `docs/runbooks/testing-strategies.md` rewritten to cover unit / integration / async / property test patterns. **PROMPT edition**: a one-line `@test` / `@property` summary added to the Attributes section so LLMs can invoke the framework without the full spec in context. **§17 Design Decisions Log** adds a new v0.5.5 row capturing the four maintainer-locked choices (Rust-shape assertions, panic + per-test isolation, full property surface in v0.5.5, async/actor tests via `@test async fn`) and the principled deferrals (`@bench`, doc tests, on-chain test scaffolding). No grammar changes (the `@property` attribute reuses the existing §16 attribute production). No new keywords (39 unchanged). Two new diagnostic registry slots reserved (`E1410` test-only-intrinsic-outside-test; `E1411` test-only-prelude-outside-test) — concrete messages earned when the compiler lands per §18.4 Growth policy. **Deferred**: `@bench` for benchmarking (different design space — warm-up, timer choice, statistical reporting); doc tests (requires a documentation-extraction pass); on-chain test scaffolding (`@onchain_test` with simulated `ctx`); test fixtures / `@before` / `@after` (currently handled by per-test setup functions). |
| v0.5.4 | **Cooperative actor termination — `handle.stop()` / `handle.kill()`** (new §8.2a; mechanics rippled through §8.1a, §8.2, §8.5, §8.7, §8.7a, §8.8, §8.10.1, §8.11). Closes issue #15 and review action P2 / item #4 (High). Resolves the orphaned-actor leak called out in the PR #9 review by introducing explicit cooperative termination as the missing fifth path to `DEAD`. **New `DRAINING` state** (§8.1a) sits between `READY` and `DEAD`: the actor handles messages already enqueued in FIFO order but rejects new sends from the moment the termination flag is set. The four-state lifecycle is `INITIALIZING → READY → DRAINING → DEAD`, with `READY → DEAD` still the failure path. **Two methods, two semantics**: `handle.stop() -> Result<(), StopError>` requests a graceful drain — the actor finishes the messages already in its mailbox, then transitions `DRAINING → DEAD`. `handle.kill() -> Result<(), StopError>` aborts after the **current handler** completes — Sploosh does not interrupt user code mid-handler, so any in-flight `.await` runs to completion before the remainder of the mailbox is discarded. `kill()` while `DRAINING` is a valid **upgrade** that returns `Ok(())`. **`StopError` enum** (new, §8.8): `AlreadyStopping` (re-stop while already stopping) and `AlreadyDead` (target is already `DEAD` or already-killed). Repeat-stop and repeat-kill are observable, not silent — the §6.1 "every fallible operation is `Result`" rule applies even though the only failure modes are these two. **Supervisor interaction** (§8.7): a child terminated via `stop()`/`kill()` is **intentional termination, not failure** — the supervisor does not restart it, and the termination does not count toward `max_restarts`. `rest_for_one` and `one_for_all` do not cascade for user-driven termination. **Handle drop semantics** (§8.2): rewritten — handle drop still does not kill the actor (non-refcounted handles are intentional), but the orphan-leak workaround now exists. Five termination paths replace the previous three: cooperative stop, immediate kill, runtime failure, supervisor decision, runtime shutdown. **Receiver convention**: method on `Handle<T>`, not a `stop` keyword (the v0.4.3 deferred `stop c` form was rejected to keep the keyword count at 39 and avoid any grammar change). Both methods are `&self` because the handle is never mutated; multiple clones racing to stop the same actor serialize on the per-actor 2-bit termination flag (`Running` / `StopRequested` / `Killed`). **Out-of-band delivery**: the termination flag is set via atomic CAS that bypasses the mailbox entirely — `stop()`/`kill()` never block on backpressure, never consume mailbox capacity, and do not interact with the §8.11 per-sender FIFO ordering. This is the only out-of-band signal in the actor model. **Self-stop / self-kill** (§8.10.1, §8.2a): legal, **not** a re-entrant call, **not** an `ActorError::SelfCall`. The signal is observed after the current handler returns, so self-stop never deadlocks. **Drop semantics** (§8.7a): unchanged — Drop on state fields runs identically when the cause is `stop()`, `kill()`, runtime failure, or supervisor restart. **Existing error variants suffice for already-rejected messages**: `SendError::Dead` (§8.5) covers `send_timeout` to a `DRAINING` actor, `ActorError::Dead` (§8.8) covers request/reply, and `send` silently drops — no new error variants on these enums. **`SendError::Dead` documentation extended** (§8.5) to call out the `DRAINING` case. **§13.0 Compiler Intrinsics**: two new rows for `Handle<A>.stop()` and `Handle<A>.kill()`, signature `fn(&Handle<A>) -> Result<(), StopError>`, native/wasm only. Both are method-call lowerings; no syntactic novelty. **On-chain availability**: rides on the existing §11.1 / §12.3 actor prohibition — `Handle<T>` itself is already a compile error inside `onchain` modules, so `stop()`/`kill()` are too. **`PROMPT` edition**: the existing `Lifecycle: INITIALIZING → READY → DEAD` line updated to four states with `DRAINING`; the "Handle drop does NOT kill the actor" line rewritten to point at the five termination paths; a one-line `handle.stop()` / `handle.kill()` summary added. **Guides updated**: `docs/guide/actors-and-concurrency.md` adds a "Stopping Actors" subsection after the existing handle-drop paragraph and rewrites the orphan-actor sentence to cross-link `stop()`. `docs/guide/ownership-and-borrowing.md` extends the "`Handle<T>` is not reference-counted" paragraph to mention the explicit termination methods. **Migration guides**: `docs/migration/from-rust.md` updates the `Arc::strong_count` row to reference `handle.stop()` as the explicit cleanup path Rust's `Arc<T>` drop has no analog for; `docs/migration/from-elixir.md` updates the PID-lifetime row to reference `handle.stop()` as Sploosh's equivalent of `Process.exit(pid, :normal)`. **§17 Design Decisions Log** adds a new v0.5.4 row capturing the method-form choice, the dual-method choice, the supervisor-as-intentional-termination choice, and the out-of-band signaling rationale. No grammar changes. No new keywords (39 unchanged). No new `docs/reference/compiler-errors.md` entries — `StopError` is a library enum, not a compiler diagnostic, and per §18.4 / Growth policy no codes are pre-assigned. **Deferred to a future amendment**: `stop_all()` on a supervisor (cohort-stop convenience), explicit "wait for DEAD" awaitable, handler-interruption semantics (POSIX-style cancellation). |
| v0.5.3 | **Manifest specification fleshed out** (§14.1–§14.4 expanded; existing §14.1 stub replaced). Closes issue #14 (slice 1 of 7 in the v0.5.3–v0.5.9 sequence) and review action U2. Spec previously had 19 lines on `sploosh.toml`; the manifest is the first artifact a future compiler will load, so locking the contract before codegen makes assumptions is the cheapest form of debt prevention. **§14.1.1 `[project]`** formalizes `name` / `version` / `edition` (required) and `description` / `license` / `authors` / `repository` (optional); unknown fields are a hard error. **`edition` is the Sploosh language version** (`"0.5"`) — pre-1.0 cadence makes year strings (`"2026"`) misleading, and tying the edition to the shipped spec version aligns release artifacts. All existing `edition = "2026"` examples updated to `"0.5"` across `docs/spec-plans/`, `docs/tooling/`, `docs/guide/`, runbooks, and examples. **§14.1.2 dependency tables** introduce `[dev-dependencies]` (test-only, not forwarded to dependents) and `[build-dependencies]` (parsed and reserved for future build-script support; no invocation specified yet) alongside `[dependencies]`. Inline-table dep form documents `version`, `features`, `default-features`, `optional`, `git` + required `rev` (branches and tags rejected as non-reproducible floats), `path` (workspace-internal only). Source precedence `path > git > registry`; multiple sources on a single entry is a manifest error. **§14.1.3 `[features]`** adopts Cargo's modern syntax: `"name"` for local feature, `"crate/feature"` for transitive feature, `"dep:crate"` for explicit optional-dep activation (resolves the Cargo-2018 ambiguity). **§14.1.4 `[target.<target>.dependencies]`** — Cargo-style per-target dep sections (one per `native`/`wasm`/`evm`/`svm`); merged additively with `[dependencies]`; on-chain prohibitions (§11.1, §12.3) still apply. **§14.1.5 `[targets]`** clarifies the existing project-level `default` / `contracts` table is distinct from §14.1.4 — different role, different shape. **§14.1.6 `[profile.<name>]`** specifies four built-in profiles (`dev`, `release`, `test` inheriting from `dev`, `bench` inheriting from `release`) and custom profiles via `inherits`. Profile knobs: `opt-level` (`0`–`3`, `"s"`, `"z"`), `lto` (`false`/`"thin"`/`"fat"`), `debug` (`0`/`1`/`2`/`false`), `strip` (`"none"`/`"debuginfo"`/`"symbols"`), `incremental` (bool), `overflow-checks` (bool). **`overflow-checks` is frozen `true` for `evm` and `svm` targets**, overriding any user setting and emitting warning `W0xxx` (registry slot reserved, no entry assigned per §18.4 Growth policy). **`codegen-units` and `panic = "abort"|"unwind"` are deliberately not exposed** — the former leaks an LLVM-specific implementation detail; the latter has no choice to make under §4.8's fixed failure model (no unwind path). Per-target profile overrides (e.g., `[profile.release.evm]`) are deferred to a future amendment. **§14.1.7 `[runtime]`** consolidates the previously inline-mentioned `threads = N` knob with a new `mailbox_default_capacity` knob; both silently ignored on `evm` / `svm` (no Sploosh-level on-chain runtime). **§14.1.8 resolution semantics** — Cargo version requirement syntax (caret/tilde/exact/comparison/wildcard), resolver v2 unification (dev-dep features kept separate from non-test feature graphs), structural conflict detection, package-scoped editions. **§14.2 Workspaces** — root `[workspace]` manifest with no `[project]` (root is not buildable); `members` (globs allowed), `exclude`, `resolver = "2"` (required, future-proofing), `[workspace.package]` and `[workspace.dependencies]` for member inheritance via `field.workspace = true`; one `sploosh.lock` at workspace root, member lockfiles rejected. **§14.3 Lockfile (`sploosh.lock`)** — TOML, `[[package]]` array entries with `name` / `version` / `source` / `checksum` / `dependencies`. **Hash algorithm: Blake3** (already present in `std::crypto` on all four backends; faster than SHA-256 for typical 2KB–100KB lockfile sizes); 32-byte digest in RFC 4648 base32 without padding, prefixed `"blake3:"`. Deterministic ordering (alphabetical by name, then version), LF line endings, schema `version = 1`. Update semantics: `sploosh build`/`test`/`check` *verify only*, never write — manifest-incompatible lockfile fails the build with reserved diagnostic slot `E14xx` (no entry assigned per §18.4 Growth policy); `sploosh update` is the only command that may rewrite the lockfile. **§14.4 dependency sources** — registry (default; URL/auth/publishing flow deferred to v0.6+), git (with required `rev` SHA), path (workspace-internal only). **Tooling mirrors**: `docs/tooling/sploosh-toml.md` rewritten as the canonical schema mirror with TODO removed; `docs/tooling/build-system.md` adds `--profile <name>` and `--target <t>` flag rows, `sploosh update` and `sploosh tree` commands; `docs/tooling/package-management.md` rewritten with sources / version syntax / lockfile model and TODO removed (registry / publishing remain marked deferred). **Runbooks**: `new-project-setup.md` updated to the v0.5.3 schema with a workspace-bootstrap variant; `cross-target-builds.md` adds a `[target.wasm.dependencies]` worked example; `adding-onchain-module.md` updates the `[targets]` snippet and cross-links to the §14.1.6 `overflow-checks` on-chain freeze. **Guide**: `getting-started.md` updates `edition` to `"0.5"`. **PROMPT edition**: a one-line manifest summary added before the existing `## File ext` footer (Cargo-shape; `[dev-dependencies]`; `[target.X.dependencies]`; four built-in profiles; Blake3 lockfile). **§17 Design Decisions Log** adds a new v0.5.3 row capturing the four user-locked choices (Blake3, Cargo-exact profiles, `[target.X.dependencies]` sections, edition = language version) and the principled omissions (`codegen-units`, `panic`). No grammar changes. No new keywords (39 unchanged). No new diagnostic registry entries — overflow-check freeze warning and lockfile-mismatch error are *reserved-slot* references only, earned when the compiler lands per §18.4. **Deferred**: registry endpoint and publishing workflow (v0.6+), build-script invocation (only `[build-dependencies]` reserved), per-target profile overrides, lockfile-less library default. |
| v0.5.2 | **`Shared<T>` immutable-refcounted primitive** (new §4.4a). Closes PR #9 review Blocker P1 — the shared-immutable-data gap that previously forced clone-everything, actor-wrap, or local-`&T`-only patterns for read-heavy data. New §4.4a "Shared Immutable Data with `Shared<T>`" defines an atomically refcounted pointer to an immutable `T`: `Shared::new(value) -> Shared<T>`; `Clone` bumps the atomic refcount O(1) with no allocation and no `T::clone` call; deterministic drop of the inner `T` when the last clone goes out of scope (preserves the "no GC" guarantee of §3.10). **Strictly less than Rust's `Arc<T>`**: immutable only (no `&mut *shared`, no `get_mut`, no `make_mut`, no `try_unwrap`); no `Weak<T>` (cycles impossible by construction because Sploosh has no `Cell` / `RefCell` / `UnsafeCell` / user-visible atomics); not `Copy` (explicit `.clone()` preserves the cost-signal of each refcount bump). **Strict `T: Send + Sync` requirement**: `Shared<T>` is `Clone + Send + Sync` iff `T: Send + Sync`, otherwise `Shared::new` is a compile error — the type exists to cross thread and actor boundaries so requiring thread-safe inner values is an enforced invariant, not a convention. **Deref semantics**: `*shared` produces `&T` only; unlike `Box<T>`'s `*boxed`, it can never move the inner value out. **Actor interop** (§8.2 addition): `Shared<T>` satisfies the §8.2 owned-parameter rule for `&mut self` methods (the wrapper moves; the inner data is shared via refcount bump), making it the idiomatic way to pass read-heavy data to actor handlers and the idiomatic reply type for `&self` request/reply methods returning cached data. **Not available on-chain** — a compile error inside `onchain` modules per both §11.1 and §12.3; reference counting has no gas or storage meaning, and every on-chain value is scoped to the transaction frame. **Drop-order clarification** (§3.10): the `Shared<T>` wrapper drops in scope-reverse order as usual; the inner `T` drops only when the last live clone goes out of scope, which may be earlier or later than any individual wrapper's lifetime — still deterministic given the set of holders. **Compound Types list** (§3.2) adds `Shared<T>`. **Prelude** (§13.1) adds `Shared` after `Box`. **§4.4** rewritten to cross-reference §4.4a and §8.2 instead of saying only "Use `Handle<T>` for sharing state". **§17 Design Decisions Log** adds a new v0.5.2 row (the v0.4 "No `Rc<T>`/`Arc<T>`" row is kept for chronological accuracy). **Guide updates**: `docs/guide/ownership-and-borrowing.md` rewrites the "No Rc/Arc" section with the two-primitive narrative (`Shared<T>` for immutable, `Handle<T>` for mutable, pick by intent); `docs/guide/actors-and-concurrency.md` updates its `Rc`/`Arc` mention to cross-reference `Shared<T>` and adds a worked example of `Shared<LookupTable>` crossing actor boundaries instead of actor-wrapping a read-heavy cache. **Migration update**: `docs/migration/from-rust.md` rewrites three rows (`Arc<Mutex<T>>`, `Rc<T>`/`Arc<T>`, `Arc::strong_count`) to contrast `Shared<T>` (immutable reads) and actor + `Handle<T>` (mutable writes). **PROMPT edition**: `Shared<T>` added to the Compounds line and the Ownership `Box<T>` bullet is rewritten to include the `Shared<T>` summary. No grammar changes. No new keywords (39 unchanged). No new `docs/reference/compiler-errors.md` entries — per §18.4 and the v0.5.1 Growth policy, Shared-specific diagnostic codes are earned when the compiler lands, not pre-assigned. |

---

*Working title: Sploosh. Name subject to change.*
*This spec is a living document. v0.5.10-draft — May 2026.*
