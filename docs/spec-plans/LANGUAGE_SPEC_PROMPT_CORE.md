# SPLOOSH Quick Reference — Core (v0.5.10) — LLM System Prompt Edition

Sploosh: AI-native language. Rust safety + Elixir concurrency + web3 targeting.

> For on-chain semantics (§11), load `LANGUAGE_SPEC_PROMPT_WEB3.md` alongside this file.

## Syntax Core
- Blocks: `{ }` — Functions: `fn` — Bindings: `let` / `const` — Types: `name: Type`
- Match: `match val { Pat => expr, }` — Pipe: `expr |> fn` — Error prop: `expr?`
- Cast: `expr as Type` (numeric only) — Visibility: `pub` or private.
- No null, no exceptions, no operator overloading, no implicit conversions, no unsafe.

## Types
Primitives: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 u256 bool char str String Address ()`. Compounds: `[T; N]` `Vec<T>` `Map<K,V>` `Set<T>` `Box<T>` `Shared<T>` `(T, U)` `Option<T>` `Result<T, E>`. Channels: `Channel<T>` `Sender<T>` `Receiver<T>`.
Custom: `struct Name { field: Type }` / `enum Name { A, B(T), C { x: T } }`. Generics: `fn name<T: Bound>(x: T) -> T { }`. Traits: `trait Name { type Item; fn method(&self) -> T; }` / `impl Trait for Type { }`. Supertraits: `trait Loggable: Printable { }` (implementors must impl both). Dynamic dispatch: `&dyn Trait`, `Box<dyn Trait>` for heterogeneous collections.

## Standard Traits
Marker: `Copy`, `Send`, `Sync`. Derivable: `Clone`, `Debug`, `Display`, `Eq`, `Ord`, `Hash`, `Serialize`, `Deserialize` (Display derive mirrors Debug shape). Conversion: `From<T>`, `Into<T>`, `TryFrom<T>`, `TryInto<T>`. Error/cleanup: `Error: Display`, `Drop` (mutually exclusive with `Copy`). Closures: `Fn`, `FnMut`, `FnOnce`. Iterators: `Iter { type Item; }`, `FromIter`.

## Type Rules
- All match/if arms must return the same type.
- Pattern bindings: primitives copy, non-Copy types move, `ref` to borrow.
- Default integer: `i64`. Default float: `f64`. Suffix to override: `42u32`, `3.14f32`.
- Local inference only. Function signatures must be fully annotated.
- `as` for numeric casts only: `x as i64`. Narrowing truncates. No non-numeric casts.

## Ownership
- Single owner. Move by default. Primitives copy. `&T` immutable borrow, `&mut T` mutable borrow. One `&mut` XOR many `&`.
- `Box<T>`: heap-allocated single-owner. `Drop` for cleanup.
- `Shared<T>`: atomically refcounted, immutable-only. `Clone + Send + Sync` iff `T: Send + Sync`. No `&mut *`, no `Weak`, no cycles. Not `Copy`. Cross-actor read-heavy data. Forbidden on-chain. No `Rc`/`Arc`.
- Lifetimes: required when returning a reference with multiple ref params. Single-source elision: `fn name(&self) -> &str` needs none. Multiple sources explicit: `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`.
- No `static` mutable state — all mutable state lives in actors. `const` supports literals, arithmetic, and known constructors only.

## Integer Overflow
Checked arithmetic everywhere by default. Overflow = actor death / program abort. `wrapping_add` (wraps), `saturating_add` (clamps), `checked_add` (returns `Option`) for explicit overflow control. `@overflow(wrapping)` opts a function into wrapping (compile error on-chain). On-chain: always checked, no exceptions.

## Math
Method syntax on numeric types; math methods are compiler intrinsics lowering to LLVM intrinsics (folding, vectorization, sin+cos fusion).

**Float methods on `f32`/`f64`** (**compile error on-chain**; float *values* and comparisons stay legal): classification (`is_nan`/`is_finite`/`is_infinite`/`is_normal`/`is_sign_positive`/`is_sign_negative`/`classify`→`FpCategory`); sign+rounding (`abs`/`signum`/`copysign`/`floor`/`ceil`/`round`/`trunc`/`fract`); `min`/`max`/`clamp`; power/root (`sqrt`/`cbrt`/`powi`/`powf`/`hypot`/`recip`); exp/log (`exp`/`exp2`/`exp_m1`/`ln`/`ln_1p`/`log`/`log2`/`log10`); trig (`sin`/`cos`/`tan`/`asin`/`acos`/`atan`/`atan2`/`sin_cos`); hyperbolic (`sinh`/`cosh`/`tanh`/`asinh`/`acosh`/`atanh`); `mul_add` (correctly rounded FMA); `to_degrees`/`to_radians`.

**Float constants** on `f32`/`f64`: `PI`, `TAU`, `E`, `SQRT_2`, `LN_2`, `LN_10`, `LOG2_E`, `LOG10_E`, `INFINITY`, `NEG_INFINITY`, `NAN`, `MIN`, `MAX`, `MIN_POSITIVE`, `EPSILON`.

**Integer methods** (all targets, on-chain OK): `abs` (signed), `min`, `max`, `clamp`, `pow` (checked), `isqrt`, `ilog2`, `ilog10`, `count_ones`/`count_zeros`/`leading_zeros`/`trailing_zeros`, `rotate_left`/`rotate_right`, `swap_bytes`, `to_be`/`to_le`/`from_be`/`from_le`.

**`@fast_math(flags)`** — LLVM fast-math: `contract`, `afn`, `reassoc`, `arcp`, `nnan`, `ninf`, `nsz`. Bare `@fast_math` = `(contract, afn)` (FMA fusion + approximate transcendentals). Per-function, not inherited. **Compile error on-chain.**

## Closures
Capture by usage: `&T` (read), `&mut T` (modify), `move` (take ownership). `move` required for `spawn` or function-return. Traits: `Fn` (borrow), `FnMut` (mut borrow), `FnOnce` (move, call once).

## Error Handling
```sploosh
fn load(path: &str) -> Result<Config, AppError> {
    let data = fs::read(path)?;      // ? propagates Err
    let cfg = json::parse(&data)?;
    Ok(Config::from(cfg))
}
```
Always `Result<T, E>` or `Option<T>`. No throw/catch. No null. `@error` on enum generates `From`, `Display`, `Error` impls. `.context("msg")` on Result wraps with context.

## Pipe + Error Rules
`expr |> f?` parses as `(expr |> f)?` = `f(expr)?`. Use `?` per fallible stage:
```sploosh
let r = input |> parse? |> validate? |> transform?;
```
Multi-arg: `x |> f(a)` = `f(x, a)`. Piped value is always first arg.

## Iterators
`Iter` trait: `type Item; fn next(&mut self) -> Option<Self::Item>`. Lazy. Adaptors: `map`, `filter`, `flat_map`, `take`, `skip`, `zip`, `chain`, `enumerate`. Terminals: `collect`, `fold`, `for_each`, `count`, `any`, `all`, `find`, `first`, `sum`. `.iter()` borrows, `for x in val` moves, `.iter_mut()` borrows mutably.

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
`Handle<T>`: Clone + Send. `send` only on `&mut self` (compile error on `&self`). `&mut self` pub params must be **owned**; `&self` pub params may take refs (caller blocks).
Lifecycle: `INITIALIZING → READY → DRAINING → DEAD` (DRAINING via `stop()`; failure skips DRAINING). `init` is infallible and non-async; panic → DEAD. Handle drop does NOT kill the actor. Five termination paths: `stop()` (drain), `kill()` (immediate, after current handler), runtime failure, supervisor decision, runtime shutdown.
`handle.stop()` / `handle.kill() -> Result<(), StopError>` (`AlreadyStopping`/`AlreadyDead`); both `&self`, out-of-band (bypass mailbox, never block). Supervisor treats stop/kill as **intentional, not failure** (no restart, no `max_restarts` hit). `kill()` upgrades `DRAINING`. Stop/kill on `INITIALIZING` latch (observed at `init`-returns).
Self request/reply → `Err(ActorError::SelfCall)`; self-sends and self-stop/kill are legal. Dead/DRAINING: `send` drops, `send_timeout` → `Err(SendError::Dead)`, request/reply → `Err(ActorError::Dead)`. Blocked senders wake on death; no transparent redirect after restart.
`select { msg = rx.recv() => handle(msg), _ = timeout(5000) => err() }` — arms top-to-bottom deterministic.

## Observability (§8.12)
Always-on, every build mode. **Direct on `Handle<T>`** (constant-time, infallible, work on dead handles): `mailbox_len()`, `mailbox_capacity()`, `alive()`, `actor_id() -> ActorId`. **Module `std::actor::observe`**: `actor_info(&handle) -> Option<ActorInfo>`, `actors() -> Iter<ActorInfo>` with `.by_supervisor(&sup)` / `.by_name(name)`. **Supervisor-rooted** (on `@supervisor` `Handle<S>`): `sup.restart_count(&child)`, `sup.restart_history(&child)` (cap 16, tune via `@supervisor(restart_history: N)`), `sup.children()`. `ActorId`: opaque `Copy + Eq + Hash`, monotonic, never reused, not `Send` across runtimes. `ActorInfo { id, name, spawn_location, supervisor: Option<ActorId>, lifecycle_state, mailbox_len, mailbox_capacity, death_cause: Option<DeathCause> }`. `DeathCause`: `RuntimeFailure { panic } | Stopped | Killed | Supervised { restart_pending } | RuntimeShutdown`. Dead-actor snapshot retained until last `Handle<T>` clone drops. Compile error inside `onchain`.

## Channels
```sploosh
let (tx, rx) = Channel::new(100);   // bounded MPSC
tx.send(val)?;                       // blocks if full
let msg = rx.recv()?;                // blocks until available
```
`Sender<T>`: Clone + Send. `Receiver<T>`: not Clone. Single consumer.

## Runtime
M:N work-stealing scheduler, one thread per core, lock-free bounded mailboxes. Per-sender FIFO ordering. Default mailbox 1024 (`@mailbox(capacity: N)`). `spawn async { expr }` for non-actor tasks; `.await` allowed in actors. Handler is busy (mailbox locked) across every `.await` — re-entrant self-call → `SelfCall`; multi-actor cycles undetected. Blocking FFI in handlers is a compile error; handler-safe FFI must be `extern "C" async`. Supervision: `@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60)` (sliding window). Restart runs fresh `init` with stored args; old handles become permanently dead (no transparent redirect). Runtime starts with `main()`, shuts down when `main()` returns.

## Async
```sploosh
async fn fetch(url: &str) -> Result<Response, NetError> {
    let r = net::get(url).await?;
    Ok(r)
}
```

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
File resolution: `mod foo;` → `foo.sp` or `foo/mod.sp`. Orphan rule: impl trait for type only if you own the trait or the type.

## Attributes & Derives
`@test` `@property` `@derive(Serialize, Clone, Debug)` `@inline` `@error` `@payable`
`@supervisor(strategy: "one_for_one")` `@mailbox(capacity: 2048)` `@overflow(wrapping)`
`@fast_math(contract, afn)` (compile error on-chain)
`#[target(evm)]` `#[cfg(test)]`
Derives: `Debug`, `Display`, `Clone`, `Copy`, `Eq`, `Hash`, `Serialize`, `Deserialize`, `Ord`.

## Testing
`@test fn name() { assert_eq(a, b); }` — zero params, returns `()` or `Result<(), TestFailure>`; may be `async`. Honored only under `#[cfg(test)]`. Each test runs in its own isolation actor; failure paths: `Err(TestFailure)` or actor death (panic). `assert_eq(a, b)` / `assert_ne(a, b)` are test-only intrinsics (`E1410` outside tests, `T: Eq + Debug`, operands borrowed as `&T`); `assert_matches(value, pattern)` only requires `Debug`. `?` propagates via `TestFailure: From<E>` for every `E: Error`. Layout: unit tests in `#[cfg(test)] mod tests`, integration tests in `tests/*.sp` (separate crate, `pub`-only). `@property fn name(x: T) { ... }` runs 256 cases with deterministic shrinking; `T: Gen` (primitives, `bool`, `String`, `Vec<T: Gen>`, `Option`, `Result`, tuples ≤12). CLI: `sploosh test [--filter pat] [--exact] [--test-threads N] [--nocapture] [--seed hex] [--cases N] [--format human|json]`. `--test-threads=1 --seed=<fixed>` is byte-deterministic. Exit codes: `0`/`1`/`2` (pass/fail/runner-error). **Compile error inside `onchain`.**

## Build
`sploosh build --target native|wasm|evm|svm`

## Diagnostics
Every diagnostic carries a stable code (`E<NNNN>` error / `W<NNNN>` warning / `L<NNNN>` lint), severity (`error`/`warning`/`help`/`note`), a primary span, and optional suggested fixes with rustc-compatible applicability (`MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, `Unspecified`). Output modes: `human` (default), `json` (NDJSON, LLM-parseable), `short` (grep-friendly). `json` records carry mandatory first field `schema: 1` (integer; bumps only on breaking change) and optional `locale` BCP-47 tag (omitted when `None` ≡ English). At most one `MachineApplicable` fix per diagnostic. Explanations: `https://sploosh.dev/errors/{code}` with lowercased code.

## Manifest (`sploosh.toml`)
Cargo-shape. `[project]` (`name`, `version`, `edition = "0.5"`), `[dependencies]`, `[dev-dependencies]` (not forwarded), `[build-dependencies]` (reserved). Inline-table form: `{ version, features, default-features, optional, git+rev, path }` — `rev` SHA required for git; `path` workspace-internal only. `[features]`: `"name"` / `"crate/feat"` / `"dep:crate"`. `[target.{native|wasm|evm|svm}.dependencies]` additive merge. Four profiles: `dev`, `release`, `test` (← `dev`), `bench` (← `release`); knobs `opt-level`/`lto`/`debug`/`strip`/`incremental`/`overflow-checks`. `overflow-checks` **frozen `true` on `evm`/`svm`**. No `codegen-units`, no `panic`. Workspaces: root `[workspace]` with `members`/`exclude`/`resolver = "2"`; members inherit via `field.workspace = true`; single `sploosh.lock` at workspace root. Lockfile: TOML, **Blake3** checksums (32-byte digest, base32-no-pad, prefix `"blake3:"`), deterministic ordering, schema `version = 1`. `sploosh build|test|check` verify only; `sploosh update` is the sole writer.

## On-chain Prohibitions
Compile errors inside `onchain`: `actor`, `spawn`, `send`, `send_timeout`, `select`, `timeout(ms)`, `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>`, `@supervisor`, `@mailbox`, `async fn`/`.await`, `extern "C"`/`extern "C" async`, every `f32`/`f64` math method, `@fast_math`, `@overflow(wrapping)`, `Shared<T>`, `std::test` (incl. `assert_eq`/`assert_ne`/`assert_matches`), `std::actor::observe`, `ActorId`, `std::{fs,net,io,db,web,env}`. Float values, fields, comparisons, and integer math stay allowed.

## File ext: `.sp` — Entry: `src/main.sp` — Manifest: `sploosh.toml`

## Common LLM Mistakes
- **Lifetimes:** with multiple ref params returning a ref, write them: `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`. Single-source elision OK (`fn name(&self) -> &str`).
- **`as` is numeric-only:** `x as i64` ✓; `x as &str` / `x as Address` ✗. Use `From`/`Into`/`TryFrom` otherwise.
- **Pattern bindings:** primitives copy, non-Copy **move** (`let Some(s) = opt` consumes `s`). Use `ref` to borrow.
- **Capitalized variants:** `None`/`Some`/`Ok`/`Err` — never lowercase.
- **Actor signatures:** `&mut self` pub methods take **owned** params; `&self` pub may take refs (caller blocks).
- **Checked arithmetic:** `+`/`-`/`*` panic on overflow by default. Use `wrapping_*` / `saturating_*` / `checked_*` for explicit semantics. `@overflow(wrapping)` per-function (off-chain only).
- **`Shared<T>` is read-only:** atomically refcounted, immutable through the pointer. No `&mut *shared`, no `Weak`, no cycles. Forbidden on-chain.
- **Pipe + `?` precedence:** `expr |> f?` parses as `(expr |> f)?` — apply `?` per fallible stage.
- **Test assertions borrow:** `assert_eq(a, b)` / `assert_ne(a, b)` take `&T` (test-only, `T: Eq + Debug`). Outside tests → `E1410`.
- **`chain::call` ergonomics:** `chain::call(addr, mod::fn, args)` returns `Result<T, ChainError>`; `?` unwraps `T`. Don't double-unwrap.
- **`u256` off-chain cost (`W0010`):** `u256` arithmetic is software-emulated on native/wasm (~10–50x slower than `u64`). Warn-by-default lint fires on arithmetic and comparison operators plus multi-instruction integer methods (canonical trigger list in `docs/reference/compiler-errors.md` and §3.1). Does not fire on declarations, params, casts, literals, or no-op-on-unsigned methods (`abs` / `min` / `max` / `clamp`). Suppress with `#[allow(W0010)]`. Not emitted on-chain.
