# std::test

> Test framework, assertions, property testing, and the `sploosh test` runner.

**Targets:** native ✅ (test only) · wasm ⚠️ (test only, single-threaded) · evm ❌ · svm ❌

`std::test` is a **compile error inside `onchain` modules** (§11.1, §12.3 of the language spec). On-chain code is tested off-chain by spawning a simulated execution context; the `@onchain_test` attribute is deferred to a future amendment.

This page mirrors §13.3 of `docs/spec-plans/LANGUAGE_SPEC.md`. The spec is the source of truth.

---

## Overview

Tests are functions annotated with `@test` (or `@property` for randomized tests). Each test runs in its own runtime-spawned isolation actor, so a single failing test never aborts the runner.

```sploosh
@test
fn add_works() {
    assert_eq(2 + 3, 5);
}

@test
async fn fetches_payload() -> Result<(), TestFailure> {
    let body = http::get("http://localhost:8080/health").await?;
    assert_eq(body, "ok");
    Ok(())
}

@property
fn reverse_reverse_is_identity(v: Vec<i32>) {
    assert_eq(v.iter().rev().rev().collect::<Vec<i32>>(), v);
}
```

Run with `sploosh test`.

---

## The `@test` attribute

A `@test`-annotated function must:

1. Take **zero parameters** (use `@property` for parameterized tests).
2. Return `()` or `Result<(), TestFailure>`.
3. Be a **free function** at module scope. `@test` on an associated function, trait method, or actor handler is a compile error.
4. Optionally be `async`. The runner spawns a fresh runtime per `async` test.

Visibility (`pub` or private) does not affect discovery — the runner finds tests by attribute, not by name.

`@test` is honored only when `#[cfg(test)]` is true (i.e., during `sploosh test`). In other build modes the function is removed by dead-code elimination after type-checking; it does not appear in the produced binary.

---

## Test layout

```
my_pkg/
├── sploosh.toml
├── src/
│   ├── lib.sp
│   └── auth.sp           # contains `#[cfg(test)] mod tests { ... }`
└── tests/
    └── login_flow.sp     # integration test crate
```

| Location | Visibility into package | Compiled as |
|---|---|---|
| `#[cfg(test)] mod tests { ... }` inside a source file | private items of the parent module | part of the same crate |
| `tests/*.sp` | only `pub` items | a separate crate per file |

`tests/` is implicitly `#[cfg(test)]` — every file inside is included only by `sploosh test`.

---

## Assertions

Three assertion intrinsics complement the existing `assert(cond, msg)`. All three are **test-only** — calling them from production code is a compile error (`E1410`).

| Intrinsic | Signature | Purpose |
|---|---|---|
| `assert_eq(a, b)` | `fn<T: Eq + Debug>(&T, &T)` | Assert `a == b`; failure message reports both values via `Debug` |
| `assert_ne(a, b)` | `fn<T: Eq + Debug>(&T, &T)` | Assert `a != b`; failure message reports both values via `Debug` |
| `assert_matches(v, p)` | `(value, pattern)` special form | Assert `v` matches the §5.2 pattern `p` |

`assert_eq` and `assert_ne` borrow their operands so they do not consume non-`Copy` values. Failure messages are produced by `Debug`, not `Display` — every type that participates in an assertion must therefore satisfy `Debug` (typically via `@derive(Debug)`).

```sploosh
@test
fn parses_expected_shape() {
    let result = parse("3 + 4");
    assert_matches(result, Ok(Expr::Add(_, _)));
    assert_eq(result.unwrap().to_string(), "(3 + 4)");
}
```

`assert_matches` uses §5.2 match-binding rules: pattern variables introduced inside the pattern are not available after the assertion (the assertion discards them).

---

## Result-shape tests with `?`

A `@test fn` declared `-> Result<(), TestFailure>` may use `?` to propagate fallible setup:

```sploosh
@test
fn loads_config() -> Result<(), TestFailure> {
    let cfg = Config::load("test.toml")?;   // fails the test if Err
    assert_eq(cfg.name, "test");
    Ok(())
}
```

`TestFailure: From<E>` is implemented for every `E: Error`, so `?` propagation is transparent. The runner reports a propagated `Err` as a test failure (distinct from an assertion failure but indistinguishable to the runner exit code).

---

## Failure semantics

Each test runs inside its own runtime-spawned isolation actor. The runner observes one of three outcomes per test:

1. **`Ok(())`** — handler returned normally. Test passes.
2. **`Err(TestFailure)`** — handler returned `Err`. Test fails; the runner records the `TestFailure`.
3. **Actor death** — handler panicked (failed `assert*`, bounds check, overflow, etc.). The runner observes `Err(ActorError::Dead)` (§8.8) and records the panic message read from the dead isolation actor's snapshot (`DeathCause::RuntimeFailure { panic }`, §8.12).

Per-test isolation means a failed test never aborts the runner. The supervisor strategy for the test cohort is conceptually `one_for_one` with `max_restarts: 0` — a failed test is recorded, not restarted.

**User-spawned actors inside tests** must be cleaned up via `handle.stop()` / `handle.kill()` (§8.2a) or rely on the runtime-shutdown sweep when the per-test isolation actor reaches `DEAD`. Under default parallel scheduling each test gets its own fresh runtime, so a leaked spawn dies with that runtime. Under `--test-threads=1` the runner reuses a single runtime across tests, so user-spawned actors that aren't explicitly stopped can leak from one test into the next — explicit cleanup is required.

---

## Async and actor tests

`@test async fn` is permitted. The runner spawns a fresh runtime per test and drives the future to completion under the same isolation actor. Channels, `select`, timeouts, and user-spawned actors all work as in production code.

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

---

## Property tests with `@property`

`@property` is a sibling to `@test` for randomized testing. A `@property fn` takes one or more parameters of types implementing `Gen`. The runner generates `N` cases (default 256), shrinks failures to a minimum reproducer, and reports both the original failing input and the shrunk minimum.

```sploosh
@property
fn reverse_reverse_is_identity(v: Vec<i32>) {
    assert_eq(v.iter().rev().rev().collect::<Vec<i32>>(), v);
}

@property(cases: 1024)
fn checked_add_never_overflows(a: u32, b: u32) {
    match a.checked_add(b) {
        Some(sum) => assert_eq(sum, a + b),    // only when no overflow
        None => assert(a > u32::MAX - b),
    }
}
```

### The `Gen` trait

```sploosh
trait Gen {
    type Item;
    fn generate(rng: &mut Rng, size: u32) -> Self::Item;
    fn shrink(value: Self::Item) -> Iter<Self::Item>;
}
```

`size` is a 0–`size_max` complexity bound the runner increases as it explores; `shrink` returns an iterator of strictly-smaller candidates the runner tries on a failed input.

**Built-in `Gen` impls** (auto-imported via the test-only prelude):

| Type | Generation strategy |
|---|---|
| `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `u256` | Uniform across the type's range; shrinks toward 0 |
| `bool` | 50/50; shrinks toward `false` |
| `f32`, `f64` | Uniform-ish across the representable range, biased toward typical values; shrinks toward 0.0 |
| `char` | Uniform across valid Unicode scalar values |
| `String` | Length scales with `size`; characters drawn from `char::Gen`; shrinks toward `""` |
| `Vec<T: Gen>` | Length scales with `size`; elements drawn from `T::Gen`; shrinks toward `[]` and toward each element's shrink |
| `Option<T: Gen>` | 50/50 `None` vs. `Some(t)`; shrinks `Some(t)` → `None`, then shrinks `t` |
| `Result<T: Gen, E: Gen>` | 50/50 `Ok` vs. `Err`; shrinks each side independently |
| Tuples up to arity 12 | Each element drawn independently; shrinks element-wise |

Implement `Gen` for your own types to test them with `@property`.

### Deterministic shrinking

Same seed reproduces the same shrunk minimum byte-for-byte. Implementations must use a deterministic shrinking schedule. Failing inputs are reported with their RNG seed, case index, and shrunk minimum.

---

## CLI reference: `sploosh test`

| Flag | Default | Purpose |
|---|---|---|
| `--filter <pat>` | none | Only run tests whose fully-qualified path matches `<pat>` |
| `--exact` | off | Treat `--filter` as exact match instead of substring |
| `--test-threads <N>` | core count | Run `N` tests concurrently (1 disables parallelism) |
| `--nocapture` | off | Forward test stdout/stderr to the terminal during the run |
| `--seed <hex>` | random | Fix the property-test RNG seed for reproduction |
| `--cases <N>` | 256 | Override the per-property case count |
| `--format <human\|json>` | human | Match `--error-format` (§18.5); JSON is one event per line |

```bash
sploosh test                                      # run all tests
sploosh test --filter parser                      # substring match
sploosh test --filter test_parses_addition --exact
sploosh test --test-threads=1 --seed=0xCAFEBABE   # deterministic
sploosh test --format json | jq '.'               # machine-readable
```

**Determinism contract.** With `--test-threads=1 --seed=<fixed>`, two runs of the same source against the same compiler version produce byte-identical output. This is the contract LLM agents and CI snapshot tests rely on.

**Exit codes.** `0` (all tests passed), `1` (any test failed), `2` (runner error: build failure, no matching tests when `--filter` was specified, etc.).

---

## Public API

| Item | Type | Purpose |
|---|---|---|
| `TestFailure` | struct | Failure record for `Result<(), TestFailure>`-shaped tests; `From<E>` for every `E: Error` |
| `Gen` | trait | Generates and shrinks values for property tests |
| `Rng` | opaque type | Deterministic random source passed to `Gen::generate`; methods `next_u32`, `next_u64`, `gen_range(min, max)`, `shuffle(&mut [T])` |
| `assert`, `assert_eq`, `assert_ne`, `assert_matches` | intrinsics | Re-exported for documentation locality (§13.3.8; `assert` is a general prelude intrinsic, the other three auto-import via the test-only prelude) |

All items are `#[cfg(test)]`-only. Referencing them outside a test build is a compile error (`E1411`).

---

## Constructing `TestFailure`

```sploosh
// From a string message
return Err(TestFailure::new("expected non-empty cache"));

// Via ? from any Error
let cfg = Config::load("test.toml")?;
```

---

## Cross-references

- §13.3 of `docs/spec-plans/LANGUAGE_SPEC.md` — full specification.
- §13.0 — `assert_eq` / `assert_ne` / `assert_matches` intrinsics.
- §13.1 — test-only prelude additions.
- §12.1 — `@test` and `@property` attributes.
- §8.2a — `handle.stop()` / `handle.kill()` for cleaning up actors spawned in tests.
- §18 (`docs/reference/compiler-errors.md`) — diagnostic codes `E1410` and `E1411`.
- `docs/runbooks/testing-strategies.md` — test patterns and recipes.
- `docs/tooling/build-system.md` — runner CLI flags.
