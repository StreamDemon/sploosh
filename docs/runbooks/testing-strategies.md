# Runbook: Testing Strategies

> Unit tests, integration tests, async/actor tests, property tests, and on-chain test patterns.

This runbook gives recipes for the common test shapes in Sploosh. The full API surface lives in [`docs/stdlib/test.md`](../stdlib/test.md); the spec is §13.3 of `LANGUAGE_SPEC.md`.

## Pre-conditions

- A working `sploosh` toolchain.
- A package with a `sploosh.toml` manifest.

## Unit Tests

Unit tests live alongside the code they exercise, inside a `#[cfg(test)] mod tests` block. They have access to the parent module's private items.

```sploosh
// src/math.sp

pub fn add(a: i64, b: i64) -> i64 { a + b }

fn checked_add_internal(a: i64, b: i64) -> Option<i64> { a.checked_add(b) }

#[cfg(test)]
mod tests {
    use super::*;

    @test
    fn test_add() {
        assert_eq(add(2, 3), 5);
    }

    @test
    fn test_internal_overflow() {
        assert_eq(checked_add_internal(i64::MAX, 1), None);
    }
}
```

Run:

```bash
sploosh test
sploosh test --filter test_add
sploosh test --filter test_add --exact
```

## Integration Tests

Integration tests live in `tests/*.sp` at the package root. Each file is compiled as its own crate and only sees the package's `pub` surface.

```
my_pkg/
├── sploosh.toml
├── src/
│   └── lib.sp
└── tests/
    ├── login_flow.sp
    └── api_smoke.sp
```

```sploosh
// tests/login_flow.sp

use my_pkg::auth;
use my_pkg::store::User;

@test
fn login_round_trip() {
    let user = User::new("alice");
    let session = auth::login(&user, "correct-horse-battery-staple")
        .expect("login should succeed for valid creds");
    assert_eq(session.user_id, user.id);
}
```

`tests/` is implicitly `#[cfg(test)]` — no `#[cfg(test)]` annotation needed at the file level.

## Result-Shape Tests with `?`

When setup is fallible, return `Result<(), TestFailure>` and use `?`:

```sploosh
@test
fn parses_config_file() -> Result<(), TestFailure> {
    let raw = fs::read_to_string("tests/fixtures/config.toml")?;
    let cfg = toml::parse(&raw)?;
    assert_eq(cfg.workers, 4);
    Ok(())
}
```

`TestFailure: From<E>` is implemented for every `E: Error`, so `?` propagation is transparent.

## Async Tests

Mark a test `async` to drive futures:

```sploosh
@test
async fn fetches_health_endpoint() -> Result<(), TestFailure> {
    let body = http::get("http://localhost:8080/health").await?;
    assert_eq(body, "ok");
    Ok(())
}
```

The runner spawns a fresh runtime per `async` test. `.await`, channels, `select`, and timeouts all work as in production code.

## Actor Tests

Tests that spawn actors must clean them up — either explicitly with `stop()`/`kill()` or by letting them die with the per-test runtime.

```sploosh
actor Counter {
    state: i64,
    fn init(start: i64) -> Self { Counter { state: start } }
    pub fn inc(&mut self, n: i64) { self.state = self.state + n; }
    pub fn get(&self) -> i64 { self.state }
}

@test
fn counter_accumulates() {
    let counter = spawn Counter::init(0);
    send counter.inc(5);
    send counter.inc(3);
    assert_eq(counter.get(), 8);
    let _ = counter.stop();   // graceful drain; mailbox is already empty
}
```

**Pattern: actor under test with mocks.** Build the actor's dependencies as fakes, inject them via `init`, drive the actor through its public methods, then assert on observable state. Expose a request/reply accessor on the actor under test that itself queries the mock — that way the test never has to synchronize across two senders.

```sploosh
// Worker exposes a `recorded_events()` accessor for testability:
//
//     actor Worker {
//         recorder: Handle<Recorder>,
//         fn init(recorder: Handle<Recorder>) -> Self { Worker { recorder } }
//         pub fn handle_job(&mut self, job: Job) { send self.recorder.record(job.name()); }
//         pub fn recorded_events(&self) -> Vec<String> { self.recorder.events() }
//         pub fn status(&self) -> Status { Status::Ready }
//     }

@test
async fn worker_processes_job() -> Result<(), TestFailure> {
    let recorder = spawn Recorder::init();
    let worker = spawn Worker::init(recorder.clone());

    send worker.handle_job(Job::new("alpha"));

    // `worker.recorded_events()` is request/reply: it blocks the test
    // until the worker handler returns, and *inside* that handler the
    // call to `recorder.events()` is request/reply from a single
    // sender (the worker). Per-sender FIFO (§8.11) then guarantees
    // the recorder has processed the worker's earlier
    // `send recorder.record(...)` before answering its own
    // `events()` query. No cross-sender race, no `timeout(ms)`
    // (that intrinsic is select-only per §8.6 / §13.0).
    let events = worker.recorded_events();

    assert_eq(events, vec!["alpha".into()]);

    let _ = worker.stop();
    let _ = recorder.stop();
    Ok(())
}
```

## Property Tests

Use `@property` with parameters whose types implement `Gen`. The runner generates 256 cases by default and shrinks failures.

```sploosh
@property
fn reverse_reverse_is_identity(v: Vec<i32>) {
    let twice: Vec<i32> = v.iter().rev().rev().cloned().collect();
    assert_eq(twice, v);
}

@property(cases: 1024)
fn checked_add_consistent(a: u32, b: u32) {
    match a.checked_add(b) {
        Some(sum) => assert_eq(sum.wrapping_sub(b), a),
        None => assert(a > u32::MAX - b),
    }
}
```

When a property fails, the runner reports both the original failing input and the shrunk minimum:

```
property reverse_reverse_is_identity FAILED at case 47
  seed:        0xCAFEBABE
  original:    [3, -1, 5, 9, -2, 0, 4]
  shrunk:      [3, -1]
```

Reproduce with `sploosh test --filter reverse_reverse_is_identity --seed=0xCAFEBABE`.

**Pattern: implement `Gen` for your own types.**

```sploosh
struct Money { amount: i64, currency: String }

impl Gen for Money {
    type Item = Money;
    fn generate(rng: &mut Rng, size: u32) -> Money {
        Money {
            amount: i64::generate(rng, size),
            currency: ["USD", "EUR", "JPY"][rng.gen_range(0, 3) as usize].into(),
        }
    }
    fn shrink(m: Money) -> Iter<Money> {
        i64::shrink(m.amount).map(move |amount| Money {
            amount,
            currency: m.currency.clone(),
        })
    }
}
```

## Determinism for CI

Snapshot-style tests and LLM agent loops require byte-identical output across runs. Use:

```bash
sploosh test --test-threads=1 --seed=0xCAFEBABE --format=json
```

With these flags, two runs of the same source against the same compiler version produce identical output.

## Filtering and Re-Running

```bash
sploosh test                                  # all tests
sploosh test --filter parser                  # substring match across paths
sploosh test --filter parser::test_addition --exact
sploosh test --filter property_               # property tests only (by convention)
```

## On-Chain Test Patterns

> **Status: deferred.** `std::test` is currently a compile error inside `onchain` modules. On-chain code is tested off-chain by spawning a simulated execution context; the `@onchain_test` attribute (with simulated `ctx`, mocked storage, and event assertions) is targeted for a later spec amendment.

In the meantime, on-chain code can be tested at the boundary by:

1. Extracting pure logic into off-chain helper modules and writing `@test` against the helpers.
2. Asserting against the shape of `chain::call(...)` callsites in unit tests of the off-chain caller.
3. Deferring full on-chain integration testing to a forge/anchor-equivalent harness that consumes the compiled artifacts.

## If Something Goes Wrong

| Symptom | Likely cause | Fix |
|---|---|---|
| `error: cannot find function 'assert_eq' in this scope` outside tests | `assert_eq` / `assert_ne` / `assert_matches` are test-only intrinsics (E1410) | Move the call into a `@test`-annotated function or a `#[cfg(test)] mod tests` block |
| `error: 'std::test' is unavailable in this context` | Referenced `TestFailure` / `Gen` / `Rng` outside a test build (E1411) | Same as above; or guard the import with `#[cfg(test)]` |
| Tests hang under `--test-threads=N` | A test leaked a spawned actor that another test interacts with | Add `let _ = handle.stop();` at end of every test that spawns; or run with `--test-threads=1` |
| Property test reports flaky failures | Non-deterministic test body (e.g., wall-clock time, network) | Tests must be deterministic given seed + input; remove non-deterministic dependencies or pin them |
| `assert_eq` reports types but not values | The type does not implement `Debug` | Add `@derive(Debug)` to the type |
| Test passes locally, fails in CI | Different `--test-threads` default | Use `--test-threads=1` in CI or fix the underlying ordering dependency |

## Related

- Spec: `docs/spec-plans/LANGUAGE_SPEC.md` §13.3.
- Stdlib: [`docs/stdlib/test.md`](../stdlib/test.md).
- Build system: [`docs/tooling/build-system.md`](../tooling/build-system.md) — `sploosh test` flag table.
- Actor cleanup: [`docs/guide/actors-and-concurrency.md`](../guide/actors-and-concurrency.md) — `Stopping Actors` section.
- Diagnostics: [`docs/reference/compiler-errors.md`](../reference/compiler-errors.md) — `E1410`, `E1411`.
