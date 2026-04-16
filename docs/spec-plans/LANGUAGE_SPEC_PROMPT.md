# SPLOOSH Quick Reference — LLM System Prompt Edition (v0.5.0)

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
- `wrapping_add` (wraps), `saturating_add` (clamps), `checked_add` (returns Option) for explicit overflow control.
- `@overflow(wrapping)` opts a function into wrapping. Compile error on-chain.
- On-chain: always checked. No exceptions.

## Math
Method syntax on numeric types. All math methods are compiler intrinsics lowering to LLVM intrinsics (enables constant folding, auto-vectorization, sin+cos fusion).

**Float methods on `f32`/`f64`** (**NOT available on-chain** — compile error):
- Classification: `is_nan`, `is_finite`, `is_infinite`, `is_normal`, `is_sign_positive`, `is_sign_negative`, `classify` → `FpCategory`
- Sign: `abs`, `signum`, `copysign`
- Rounding: `floor`, `ceil`, `round`, `trunc`, `fract`
- Min/max: `min`, `max`, `clamp`
- Power/root: `sqrt`, `cbrt`, `powi`, `powf`, `hypot`, `recip`
- Exp/log: `exp`, `exp2`, `exp_m1`, `ln`, `ln_1p`, `log`, `log2`, `log10`
- Trig: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sin_cos`
- Hyperbolic: `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
- FMA: `mul_add` (correctly rounded). Angle: `to_degrees`, `to_radians`.

**Float constants:** `f64::PI`, `f64::TAU`, `f64::E`, `f64::SQRT_2`, `f64::LN_2`, `f64::LN_10`, `f64::LOG2_E`, `f64::LOG10_E`, `f64::INFINITY`, `f64::NEG_INFINITY`, `f64::NAN`, `f64::MIN`, `f64::MAX`, `f64::MIN_POSITIVE`, `f64::EPSILON`. Same names on `f32`.

**Integer methods** (all targets **including on-chain**): `abs` (signed), `min`, `max`, `clamp`, `pow` (checked), `isqrt`, `ilog2`, `ilog10`, `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`, `rotate_left`, `rotate_right`, `swap_bytes`, `to_be`, `to_le`, `from_be`, `from_le`.

**`@fast_math(flags)`** enables LLVM fast-math flags: `contract`, `afn`, `reassoc`, `arcp`, `nnan`, `ninf`, `nsz`. Bare `@fast_math` = `@fast_math(contract, afn)` (safe subset: FMA fusion + approximate transcendentals, no NaN/Inf UB). Per-function scope, not inherited. **Compile error on-chain.**

**On-chain rule:** every `f32`/`f64` math method from §4.10 is a compile error inside `onchain` modules — even deterministic ones like `sqrt`, `min`, `abs`. Float *values* (fields, `==`/`<`/`>`, arguments) are still allowed; only method *calls* are rejected. Use integer math on-chain.

```sploosh
fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0; let dy = a.1 - b.1;
    dx.hypot(dy)                              // overflow-safe
}
let (s, c) = theta.sin_cos();                 // fuses to llvm.sincos
let n = 1000u64.ilog2();                      // on-chain OK
```

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
`Handle<T>`: Clone + Send. `send` valid only on `&mut self` methods; `send` on `&self` is a compile error.
`&mut self` pub params must be **owned**. `&self` pub params **may take references** (caller blocks, stack outlives call).
Lifecycle: `INITIALIZING → READY → DEAD`. `init` is infallible and non-async; init panic → DEAD, supervisor notified.
Handle drop does NOT kill the actor — actors die only via failure, supervisor, or runtime shutdown.
Direct self request/reply → `Err(ActorError::SelfCall)`. Self-sends (`send self.handle.method(args)`) are legal.
Dead actor: `send` drops, `send_timeout` → `Err(SendError::Dead)`, request/reply → `Err(ActorError::Dead)`.
Blocked senders wake immediately on destination death; no transparent redirect after supervisor restart.
`select { msg = rx.recv() => handle(msg), _ = timeout(5000) => err() }` — arms top-to-bottom deterministic.

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
- Actor handler is busy (mailbox locked) across every `.await` — re-entrant self-call → `SelfCall`, multi-actor cycles undetected.
- Blocking FFI in actor handlers is a compile error. Handler-safe FFI must be `extern "C" async`.
- Supervision: `@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60)` (sliding window).
- Restart: fresh `init` with stored args; old handles become permanently dead (no transparent redirect).
- Actors are native/wasm only. Compile error inside `onchain`: `actor`, `spawn`, `send`, `send_timeout`, `select`, `timeout(ms)`, `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>`, `@supervisor`, `@mailbox`, `async fn`/`.await`, `extern "C"`/`extern "C" async`.
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
SVM: `ctx::lamports()`, `ctx::program_id()`, `ctx::signer()`, `ctx::compute_units_remaining()`.
Non-reentrant by default. `onchain` cannot use: `std::fs`, `std::net`, `std::io`, `std::db`, `std::web`, `std::env`. `onchain` cannot call `f32`/`f64` math methods or use `@fast_math` (see Math section).

**Storage layout (§11.1a).** Target-pluggable; EVM adopts Solidity-compatible slots verbatim. Struct fields: sequential `u256` slots from 0 in declaration order, same-slot primitives right-aligned and packed (matches Solidity). `Map<K,V>` value at `keccak256(abi.encode(key, map_slot))` for value-type keys; nested maps recurse. `Vec<T>`: length at slot `s`, data at `keccak256(s)`. `String`: follows Solidity `bytes`/`string` (≤31-byte payloads inlined in slot `s`; longer payloads store data at `keccak256(s)`). `[T; N]` inline. SVM uses account-based storage; layout deferred to Solana amendment.

**Reentrancy guard (§11.3a).** Runtime per-contract bool flag. Set on entry to any non-`@reentrant` `pub` on-chain function; cleared on return (Ok, Err, or revert). Cross-contract callback into a guarded function → `ChainError::Reentrancy` (revert). `@reentrant` disables check+set for that function only. Distinct from §8.10.1 actor `SelfCall` — same word, different layer.

**Cross-contract calls (§11.4a).** Declare callee signatures with new syntax:
```sploosh
extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
}
let bal = chain::call(addr, token::balance_of, user)?;  // bal: u256; chain::call returns Result<T, ChainError> and `?` unwraps T
```
Sync on EVM (lowers to `CALL`). Solidity ABI for argument/return encoding. `?` propagates `ChainError::Reverted { data: Vec<u8> }` (revert data bounded by `RETURNDATACOPY`, allocated in caller's frame — same as Solidity). `ChainError = { Reverted, OutOfGas, Reentrancy, InvalidTarget, DecodingError }`. No delegatecall in v0.4.x. SVM: CPI lowering, concrete ABI deferred. **Distinct from `extern "C"` (§4.9)** — different calling convention, safety model, and error surface; not interchangeable.

**Gas model (§11.7a).** Target-pluggable. **EVM**: gas; `ctx::gas_remaining() -> u256` EVM-only; `#[gas_limit(N)]` EVM-only advisory in ABI metadata (runtime OOG from VM, not annotation); costs per active hard fork's EIPs. **SVM**: compute units; `ctx::compute_units_remaining() -> u64` SVM-only; `#[gas_limit]` compile error on SVM. **Native/wasm**: all three are compile errors. **OOG → transaction revert**: all storage mutations and emitted events unwound; revert is transaction-wide and **unaffected by per-function attributes including `@reentrant`** — guard flag is unwound on revert, so failed calls cannot leave it stuck.

**Event `#[indexed]` (§11.5).** Up to 3 indexed fields per event variant on EVM (topics 1–3; topic 0 is signature hash). More than 3 on EVM → compile error. SVM accepts `#[indexed]` as a no-op.

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
`@fast_math(contract, afn)` (compile error on-chain)
`#[target(evm)]` `#[cfg(test)]`
Derives: `Debug`, `Clone`, `Copy`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Ord`.

## Build
`sploosh build --target native|wasm|evm|svm`

## File ext: `.sp` — Entry: `src/main.sp` — Manifest: `sploosh.toml`
