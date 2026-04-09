# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sploosh is an AI-native programming language combining Rust-level safety and performance with Elixir-style actor concurrency, targeting native (LLVM), WASM, and on-chain (EVM/SVM) compilation. The language is currently in specification phase (v0.3.0-draft).

## Repository Structure

This repository currently contains the language specification documents. There is no compiler implementation yet.

- `docs/spec-plans/LANGUAGE_SPEC.md` — **The authoritative language specification.** This is the top reference document for all language design decisions.
- `docs/spec-plans/LANGUAGE_SPEC_PROMPT.md` — Condensed quick-reference version (~4000 tokens) designed for use as an LLM system prompt.

## Documentation Rule

**Documentation must be updated alongside implementation.** Any PR that adds, changes, or removes language features, compiler behavior, stdlib APIs, or tooling must include corresponding updates to the relevant docs. A PR without matching doc updates is incomplete and should not be merged. This applies to:

- `docs/guide/` — when language semantics or syntax change
- `docs/stdlib/` — when stdlib APIs are added or modified
- `docs/web3/` — when on-chain behavior changes
- `docs/reference/` — when grammar, operators, keywords, or attributes change
- `docs/tooling/` — when build system or developer tooling changes
- `docs/runbooks/` — when operational procedures are affected
- `docs/examples/` — when examples become outdated due to changes

## Git Workflow Rules

**The `main` branch must never be directly committed to or pushed to.** All changes must follow this workflow:

1. **Create a feature branch** from `main` for all work: `git checkout -b <type>/<descriptive-name>` (e.g., `feature/add-generics-spec`, `fix/actor-ownership-rules`, `docs/update-pipe-semantics`).
2. **Make commits on the feature branch** with clear, descriptive messages.
3. **Open a Pull Request** to merge into `main`. PRs require review before merging.
4. **Never force-push to `main`** or use `git push --force` on shared branches.
5. **Delete feature branches** after merging.

Branch naming conventions:
- `feature/` — new language features, spec additions
- `fix/` — corrections to existing spec content
- `docs/` — documentation improvements, formatting, examples
- `refactor/` — restructuring spec organization without changing semantics

## Sploosh Language Specification Reference

The full specification lives in `docs/spec-plans/LANGUAGE_SPEC.md`. What follows is the complete quick-reference for the language, which must be understood and followed when writing Sploosh code examples, updating the spec, or building tooling.

### Syntax Core
- Blocks: `{ }` -- Functions: `fn` -- Bindings: `let` / `const` -- Types: `name: Type`
- Match: `match val { Pat => expr, }` -- Pipe: `expr |> fn` -- Error prop: `expr?`
- Visibility: `pub` or private. No other levels.
- No null, no exceptions, no operator overloading, no implicit conversions.

### Types
Primitives: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 bool char str String ()`
Compounds: `[T; N]` `Vec<T>` `Map<K,V>` `Set<T>` `(T, U)` `Option<T>` `Result<T, E>`
Custom: `struct Name { field: Type }` / `enum Name { A, B(T), C { x: T } }`
Generics: `fn name<T: Bound>(x: T) -> T { }`
Traits: `trait Name { fn method(&self) -> T; }` / `impl Trait for Type { }`
Supertraits: `trait Loggable: Printable { }` -- implementors must impl both.
Dynamic dispatch: `&dyn Trait`, `Box<dyn Trait>` for heterogeneous collections.

### Type Rules
- All match/if arms must return the same type.
- Pattern bindings: primitives copy, non-Copy types move, `ref` to borrow.
- Matching on `&self` in impls: bindings are auto-references.
- Default integer: `i64`. Default float: `f64`. Suffix to override: `42u32`, `3.14f32`.
- Local inference only. Function signatures must be fully annotated.

### Ownership
- Single owner. Move by default. Primitives copy.
- `&T` immutable borrow, `&mut T` mutable borrow. One `&mut` XOR many `&`.
- Lifetimes explicit: `fn f<'a>(x: &'a str) -> &'a str`
- No `static` mutable state. All mutable state lives in actors.
- `const` supports literals, arithmetic, and known constructors only.

### Closures
Capture by usage: `&T` (read), `&mut T` (modify), `move` (take ownership).
`move` required when closure is passed to `spawn` or returned from function.
Traits: `Fn` (borrow), `FnMut` (mut borrow), `FnOnce` (move, call once).
```sploosh
let data = vec![1, 2, 3];
let f = move || process(data);  // data moved into closure
```

### Error Handling
```sploosh
fn load(path: &str) -> Result<Config, AppError> {
    let data = fs::read(path)?;      // ? propagates Err
    let cfg = json::parse(&data)?;
    Ok(Config::from(cfg))
}
```
Always `Result<T, E>` or `Option<T>`. No throw/catch. No null.

`@error` on enum generates `From`, `Display`, `Error` impls:
```sploosh
@error
enum AppError {
    NotFound { resource: String },
    Database(DbError),    // auto: impl From<DbError>
}
```
`.context("msg")` on Result wraps errors with context string.

### Pipe + Error Rules
`expr |> f?` parses as `(expr |> f)?` = `f(expr)?`. Use `?` per fallible stage:
```sploosh
let r = input |> parse? |> validate? |> transform?;
```
Multi-arg: `x |> f(a)` = `f(x, a)`. Piped value is always first argument.
No placeholder syntax. Use closures for other positions: `x |> (|v| f(a, v))`.

### Destructuring & Pattern Matching
```sploosh
let (x, y) = get_point();              // tuple destructuring
let User { name, age, .. } = user;     // struct destructuring, .. for rest
if let Some(val) = maybe_val { use(val); }  // single-pattern match
while let Ok(msg) = conn.read() { handle(msg); }
```

### Iterators
`Iter` trait: `fn next(&mut self) -> Option<Item>`. Lazy by default.
Adaptors: `map`, `filter`, `flat_map`, `take`, `skip`, `zip`, `chain`, `enumerate`.
Terminals: `collect`, `fold`, `for_each`, `count`, `any`, `all`, `find`, `first`, `sum`.
```sploosh
let names = users.iter()
    |> filter(|u| u.active)
    |> map(|u| u.name.clone())
    |> collect::<Vec<String>>();
```
`.iter()` borrows, `for x in val` moves, `.iter_mut()` borrows mutably.

### Format Strings
`format("template {}", val)` -- compiler intrinsic. Specifiers: `{}` display,
`{:?}` debug, `{:.2}` float precision, `{:x}` hex, `{:>10}` align. No f-strings.

### Strings
Methods on `str`: `len`, `trim`, `contains`, `starts_with`, `split`, `replace`,
`to_uppercase`, `to_lowercase`, `find`, `chars`.
`String` adds: `push_str`, `push`, `clear`. No `+` concat -- use `format()` or `push_str`.

### Control Flow
```sploosh
if cond { a } else { b }           // expression
match val { Ok(v) => v, Err(e) => return Err(e) }
for item in list { }                // iteration
while cond { }                      // conditional loop
loop { break; }                     // infinite
data |> transform |> validate       // pipe
```

### Concurrency
```sploosh
actor Counter {
    state: i64,
    fn init(n: i64) -> Self { Counter { state: n } }
    pub fn inc(&mut self, n: i64) { self.state = self.state + n; }
    pub fn get(&self) -> i64 { self.state }
}
let c: Handle<Counter> = spawn Counter::init(0);
send c.inc(5);           // fire-and-forget (&mut self)
let val = c.get();       // request/reply, blocks (&self)
```
`Handle<T>`: Clone + Send. Store in structs, pass between actors.
Actors can be generic: `actor Cache<K: Hash + Eq + Send, V: Clone + Send> { ... }`
Actor pub method params must be **owned types** (no `&` references -- messages are async).
Dead actor: request/reply returns `Err(ActorError::Dead)`. `send` silently drops.
Actors die from runtime checks (bounds, overflow, assert). Supervisors restart.
`select { msg = recv ch => handle(msg), _ = timeout(5000) => err() }`

### Async
```sploosh
async fn fetch(url: &str) -> Result<Response, NetError> {
    let r = net::get(url).await?;
    Ok(r)
}
```

### Web3
```sploosh
onchain mod token {
    storage { balances: Map<Address, u256>, supply: u256 }
    pub fn transfer(to: Address, amt: u256) -> Result<(), TokenError> {
        let sender = ctx::caller();
        let bal = storage::get(&self.balances, sender)?;
        if bal < amt { return Err(TokenError::InsufficientBalance); }
        storage::set(&mut self.balances, sender, bal - amt);
        storage::set(&mut self.balances, to, storage::get(&self.balances, to)? + amt);
        emit Transfer { from: sender, to, amt };
        Ok(())
    }
}
```
`ctx` API: `caller()`, `self_address()`, `timestamp()`, `block_number()`.
EVM: `ctx::value()` (requires `@payable`), `ctx::gas_remaining()`, `ctx::chain_id()`.
SVM: `ctx::lamports()`, `ctx::program_id()`, `ctx::signer()`.
`@payable` for functions receiving native tokens. Non-reentrant by default.
Cross-contract: `chain::call(addr, module::fn, args)?`
`onchain` modules cannot use: `std::fs`, `std::net`, `std::io`, `std::db`, `std::web`, `std::env`.

### Modules
```sploosh
mod auth { pub mod login; pub mod token; }
use std::collections::Map;
use crate::models::{User, Role};
```

### Attributes & Derives
`@test` `@derive(Serialize, Clone, Debug)` `@inline` `@error` `@payable`
`@supervisor(strategy: "one_for_one")` `#[target(evm)]` `#[cfg(test)]`
Derives: `Debug`, `Clone`, `Copy`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Ord`.

### Build
`sploosh build --target native|wasm|evm|svm`

### File Conventions
- File extension: `.sp`
- Entry point: `src/main.sp`
- Library root: `src/lib.sp`
- Project manifest: `sploosh.toml`
- Module files: `mod.sp` within directories

### Keywords (38 total)
**Declarations:** `fn` `let` `const` `type` `struct` `enum` `trait` `impl` `mod` `use` `pub`
**Control Flow:** `if` `else` `match` `for` `in` `while` `loop` `break` `continue` `return`
**Types & Values:** `self` `Self` `true` `false` `none`
**Concurrency:** `actor` `send` `recv` `spawn` `async` `await` `select`
**Closures:** `move`
**Web3:** `onchain` `offchain` `storage` `emit`

### Design Principles
1. **One way to do everything.** Every operation has exactly one syntactic form.
2. **Familiar vocabulary only.** Every keyword/operator from top 12 most-trained languages.
3. **Explicit over implicit.** No implicit conversions, no hidden control flow.
4. **Errors are values.** All fallible operations return `Result<T, E>`.
5. **Concurrency is structural.** Actor-based isolation with message passing.
6. **Dual-target by design.** Single source compiles to native, WASM, and on-chain bytecode.
7. **Spec fits in a prompt.** Full language core describable in under 4,000 tokens.
