# SPLOOSH Quick Reference — LLM System Prompt Edition (v0.4.0)

Sploosh: AI-native language. Rust safety + Elixir concurrency + web3 targeting.

## Syntax Core
- Blocks: `{ }` — Functions: `fn` — Bindings: `let` / `const` — Types: `name: Type`
- Match: `match val { Pat => expr, }` — Pipe: `expr |> fn` — Error prop: `expr?`
- Cast: `expr as Type` (numeric only) — Visibility: `pub` or private.
- No null, no exceptions, no operator overloading, no implicit conversions, no unsafe.

## Types
Primitives: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 u256 bool char str String Address ()`
Compounds: `[T; N]` `Vec<T>` `Map<K,V>` `Set<T>` `Box<T>` `(T, U)` `Option<T>` `Result<T, E>`
Channels: `Channel<T>` `Sender<T>` `Receiver<T>`
Custom: `struct Name { field: Type }` / `enum Name { A, B(T), C { x: T } }`
Generics: `fn name<T: Bound>(x: T) -> T { }`
Traits: `trait Name { type Item; fn method(&self) -> T; }` / `impl Trait for Type { }`
Supertraits: `trait Loggable: Printable { }` — implementors must impl both.
Dynamic dispatch: `&dyn Trait`, `Box<dyn Trait>` for heterogeneous collections.

## Standard Traits
Marker: `Copy`, `Send`, `Sync`. Derivable: `Clone`, `Debug`, `Eq`, `Ord`, `Hash`, `Serialize`, `Deserialize`.
Conversion: `From<T>`, `Into<T>`, `TryFrom<T>`, `TryInto<T>`, `Display`.
Error/cleanup: `Error: Display`, `Drop` (mutually exclusive with `Copy`).
Closures: `Fn`, `FnMut`, `FnOnce`. Iterators: `Iter { type Item; }`, `FromIter`.

## Type Rules
- All match/if arms must return the same type.
- Pattern bindings: primitives copy, non-Copy types move, `ref` to borrow.
- Default integer: `i64`. Default float: `f64`. Suffix to override: `42u32`, `3.14f32`.
- Local inference only. Function signatures must be fully annotated.
- `as` for numeric casts only: `x as i64`. Narrowing truncates. No non-numeric casts.

## Ownership
- Single owner. Move by default. Primitives copy.
- `&T` immutable borrow, `&mut T` mutable borrow. One `&mut` XOR many `&`.
- `Box<T>`: heap-allocated single-owner. `Drop` trait for cleanup. No `Rc`/`Arc`.
- Lifetimes: required when returning a reference with multiple ref params.
  Single-source elision: `fn name(&self) -> &str` needs no annotation.
  Multiple sources explicit: `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`
- No `static` mutable state. All mutable state lives in actors.
- `const` supports literals, arithmetic, and known constructors only.

## Integer Overflow
- Checked arithmetic everywhere by default. Overflow = actor death / program abort.
- `wrapping_add`, `saturating_add`, `checked_add` for intentional wrapping.
- `@overflow(wrapping)` opts a function into wrapping. Compile error on-chain.
- On-chain: always checked. No exceptions.

## Closures
Capture by usage: `&T` (read), `&mut T` (modify), `move` (take ownership).
`move` required when closure is passed to `spawn` or returned from function.
Traits: `Fn` (borrow), `FnMut` (mut borrow), `FnOnce` (move, call once).

## Error Handling
```sploosh
fn load(path: &str) -> Result<Config, AppError> {
    let data = fs::read(path)?;      // ? propagates Err
    let cfg = json::parse(&data)?;
    Ok(Config::from(cfg))
}
```
Always `Result<T, E>` or `Option<T>`. No throw/catch. No null.

`@error` on enum generates `From`, `Display`, `Error` impls.
`.context("msg")` on Result wraps errors with context string.

## Pipe + Error Rules
`expr |> f?` parses as `(expr |> f)?` = `f(expr)?`. Use `?` per fallible stage:
```sploosh
let r = input |> parse? |> validate? |> transform?;
```
Multi-arg: `x |> f(a)` = `f(x, a)`. Piped value is always first argument.

## Iterators
`Iter` trait: `type Item; fn next(&mut self) -> Option<Self::Item>`. Lazy by default.
Adaptors: `map`, `filter`, `flat_map`, `take`, `skip`, `zip`, `chain`, `enumerate`.
Terminals: `collect`, `fold`, `for_each`, `count`, `any`, `all`, `find`, `first`, `sum`.
`.iter()` borrows, `for x in val` moves, `.iter_mut()` borrows mutably.

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
`Handle<T>`: Clone + Send. Actor pub params must be **owned types** (no references).
Dead actor: request/reply → `Err(ActorError::Dead)`. `send` silently drops.
`select { msg = rx.recv() => handle(msg), _ = timeout(5000) => err() }`

## Channels
```sploosh
let (tx, rx) = Channel::new(100);   // bounded MPSC
tx.send(val)?;                       // blocks if full
let msg = rx.recv()?;                // blocks until available
```
`Sender<T>`: Clone + Send. `Receiver<T>`: not Clone. Single consumer.

## Runtime
- M:N work-stealing scheduler. One thread per core. Lock-free bounded mailboxes.
- Per-sender FIFO message ordering. Bounded mailbox (default 1024). `@mailbox(capacity: N)`.
- `spawn async { expr }` for non-actor async tasks. `.await` allowed in actors.
- Supervision: `@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60)`
- Runtime starts with `main()`, shuts down when `main()` returns.

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
Non-reentrant by default. `onchain` cannot use: `std::fs`, `std::net`, `std::io`, `std::db`, `std::web`, `std::env`.

## FFI
```sploosh
extern "C" {
    fn c_open(path: &str, flags: i32) -> Result<i32, FfiError>;
}
```
No `unsafe`. Compiler generates safe wrappers. No raw pointers.

## Modules
```sploosh
mod auth { pub mod login; pub mod token; }
use std::collections::Map;
use crate::models::{User, Role};
pub use crate::models::User;   // re-export
```
File resolution: `mod foo;` → `foo.sp` or `foo/mod.sp`.
Orphan rule: impl trait for type only if you own the trait or the type.

## Attributes & Derives
`@test` `@derive(Serialize, Clone, Debug)` `@inline` `@error` `@payable`
`@supervisor(strategy: "one_for_one")` `@mailbox(capacity: 2048)` `@overflow(wrapping)`
`#[target(evm)]` `#[cfg(test)]`
Derives: `Debug`, `Clone`, `Copy`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Ord`.

## Build
`sploosh build --target native|wasm|evm|svm`

## File ext: `.sp` — Entry: `src/main.sp` — Manifest: `sploosh.toml`
