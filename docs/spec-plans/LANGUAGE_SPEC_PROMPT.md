# SPLOOSH Quick Reference — LLM System Prompt Edition (v0.3.0)

Sploosh: AI-native language. Rust safety + Elixir concurrency + web3 targeting.

## Syntax Core
- Blocks: `{ }` — Functions: `fn` — Bindings: `let` / `const` — Types: `name: Type`
- Match: `match val { Pat => expr, }` — Pipe: `expr |> fn` — Error prop: `expr?`
- Visibility: `pub` or private. No other levels.
- No null, no exceptions, no operator overloading, no implicit conversions.

## Types
Primitives: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 bool char str String ()`
Compounds: `[T; N]` `Vec<T>` `Map<K,V>` `Set<T>` `(T, U)` `Option<T>` `Result<T, E>`
Custom: `struct Name { field: Type }` / `enum Name { A, B(T), C { x: T } }`
Generics: `fn name<T: Bound>(x: T) -> T { }`
Traits: `trait Name { fn method(&self) -> T; }` / `impl Trait for Type { }`
Supertraits: `trait Loggable: Printable { }` — implementors must impl both.
Dynamic dispatch: `&dyn Trait`, `Box<dyn Trait>` for heterogeneous collections.

## Type Rules
- All match/if arms must return the same type.
- Pattern bindings: primitives copy, non-Copy types move, `ref` to borrow.
- Matching on `&self` in impls: bindings are auto-references.
- Default integer: `i64`. Default float: `f64`. Suffix to override: `42u32`, `3.14f32`.
- Local inference only. Function signatures must be fully annotated.

## Ownership
- Single owner. Move by default. Primitives copy.
- `&T` immutable borrow, `&mut T` mutable borrow. One `&mut` XOR many `&`.
- Lifetimes explicit: `fn f<'a>(x: &'a str) -> &'a str`
- No `static` mutable state. All mutable state lives in actors.
- `const` supports literals, arithmetic, and known constructors only.

## Closures
Capture by usage: `&T` (read), `&mut T` (modify), `move` (take ownership).
`move` required when closure is passed to `spawn` or returned from function.
Traits: `Fn` (borrow), `FnMut` (mut borrow), `FnOnce` (move, call once).
```sploosh
let data = vec![1, 2, 3];
let f = move || process(data);  // data moved into closure
```

## Error Handling
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

## Pipe + Error Rules
`expr |> f?` parses as `(expr |> f)?` = `f(expr)?`. Use `?` per fallible stage:
```sploosh
let r = input |> parse? |> validate? |> transform?;
```
Multi-arg: `x |> f(a)` = `f(x, a)`. Piped value is always first argument.
No placeholder syntax. Use closures for other positions: `x |> (|v| f(a, v))`.

## Destructuring & Pattern Matching
```sploosh
let (x, y) = get_point();              // tuple destructuring
let User { name, age, .. } = user;     // struct destructuring, .. for rest
if let Some(val) = maybe_val { use(val); }  // single-pattern match
while let Ok(msg) = conn.read() { handle(msg); }
```

## Iterators
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

## Format Strings
`format("template {}", val)` — compiler intrinsic. Specifiers: `{}` display,
`{:?}` debug, `{:.2}` float precision, `{:x}` hex, `{:>10}` align. No f-strings.

## Strings
Methods on `str`: `len`, `trim`, `contains`, `starts_with`, `split`, `replace`,
`to_uppercase`, `to_lowercase`, `find`, `chars`.
`String` adds: `push_str`, `push`, `clear`. No `+` concat — use `format()` or `push_str`.

## Control Flow
```sploosh
if cond { a } else { b }           // expression
match val { Ok(v) => v, Err(e) => return Err(e) }
for item in list { }                // iteration
while cond { }                      // conditional loop
loop { break; }                     // infinite
data |> transform |> validate       // pipe
```

## Concurrency
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
Actor pub method params must be **owned types** (no `&` references — messages are async).
Dead actor: request/reply → `Err(ActorError::Dead)`. `send` silently drops.
Actors die from runtime checks (bounds, overflow, assert). Supervisors restart.
`select { msg = recv ch => handle(msg), _ = timeout(5000) => err() }`

## Async
```sploosh
async fn fetch(url: &str) -> Result<Response, NetError> {
    let r = net::get(url).await?;
    Ok(r)
}
```

## Web3
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

## Modules
```sploosh
mod auth { pub mod login; pub mod token; }
use std::collections::Map;
use crate::models::{User, Role};
```

## Attributes & Derives
`@test` `@derive(Serialize, Clone, Debug)` `@inline` `@error` `@payable`
`@supervisor(strategy: "one_for_one")` `#[target(evm)]` `#[cfg(test)]`
Derives: `Debug`, `Clone`, `Copy`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Ord`.

## Build
`sploosh build --target native|wasm|evm|svm`

## File ext: `.sp` — Entry: `src/main.sp` — Manifest: `sploosh.toml`
