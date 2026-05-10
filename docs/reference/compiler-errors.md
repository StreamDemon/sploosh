# Compiler Errors

> Stable registry of Sploosh diagnostic codes. The **format** of a diagnostic
> (record shape, severity levels, applicability vocabulary, output modes,
> stability contract) is specified in `docs/spec-plans/LANGUAGE_SPEC.md` §18.
> This file is the **registry** — the canonical `code → meaning` mapping
> that §18.4 freezes.

Each diagnostic that the compiler or runtime emits carries a stable code of
the form `E<NNNN>` (error), `W<NNNN>` (warning), or `L<NNNN>` (lint). The
`Kind` column distinguishes three classes of diagnostic: **compile error**
(raised at build time, no deployed artifact); **runtime revert** (raised
during on-chain transaction execution — the entire transaction's state
mutations and emitted events are unwound, per §11.7a); and **runtime
error** (raised by an off-chain runtime API as a `Result::Err` value,
returned to the caller for handling — actors do not have transaction-wide
rollback semantics, so an actor-runtime error never unwinds prior state).
Runtime-error and runtime-revert entries mirror variants defined in their
spec sections (§8.7, §8.8, §8.12.3, §11.3a, §11.4a, §11.7a); the variant
semantics are specified there, and this page is the canonical source of
each stable code number. The `Cluster` column locates the code in the
partitioning defined in §18.2. The `Status` column is one of `stable` (published code — the
`code → meaning` mapping is frozen per §18.4), `deprecated` (superseded —
carries a `superseded_by: <code>` note), or `reserved` (range reserved for
future assignment by cluster).

## Cluster A — Lexical / parser / basic syntax · `E0001–E0999`

Reserved range. Entries will be filled as the lexer and parser land and
earn specific codes.

<!-- TODO: Populate once §2 lexical or §16 grammar diagnostics are implemented. -->

## Cluster B — Type system, trait coherence, ownership, lifetimes · `E1000–E1099`

Reserved range. Type-checker and borrow-checker diagnostics will earn
codes in this range as they land.

<!-- TODO: Populate once type-checker diagnostics are implemented. -->

## Cluster C — On-chain · `E1100–E1199`

| Code | Kind | Cluster | Status | Message / meaning | Spec ref |
|------|------|---------|--------|-------------------|----------|
| `E1101` | runtime revert | C | stable | `ChainError::Reentrancy` — a cross-contract callback re-entered a non-`@reentrant` `pub` function of a contract whose reentrancy flag was already set. The transaction reverts and all state mutations are unwound. | §11.3a |
| `E1102` | runtime revert | C | stable | `ChainError::OutOfGas` — the callee of a `chain::call` exhausted its forwarded gas. The callee's frame reverts; the caller observes `Err(ChainError::OutOfGas)` provided it retains enough gas to handle the error path. Transaction-wide OOG reverts the entire transaction. | §11.4a, §11.7a |
| `E1103` | compile error | C | stable | Gas intrinsic used on the wrong target. `ctx::gas_remaining()` is EVM-only; `ctx::compute_units_remaining()` is SVM-only; `#[gas_limit(N)]` is EVM-only. Using any of these on an unsupported target is a compile error. | §11.7a |
| `E1104` | compile error | C | stable | Invalid `extern onchain mod` interface. Common causes: function body present (only signatures allowed), signature missing return type `Result<T, E>`, error type not in scope, block nested inside `extern "C"` or inside a function body. | §11.4a |
| `E1105` | compile error | C | stable | `#[indexed]` used outside an event variant field, or more than three `#[indexed]` fields on a single event variant compiled for EVM (topic slots 1–3 are the only indexed slots; topic 0 is reserved for the event signature hash). | §11.5 |
| `E1106` | compile error | C | stable | `@reentrant` applied to a non-`pub` or non-on-chain function. The attribute only disables the §11.3a per-contract guard; it has no meaning outside `pub` functions of an `onchain mod`. | §11.3a, §12.1 |
| `E1107` | compile error | C | stable | `extern "C"` block declared inside an `onchain` module, or `extern onchain mod` declared inside `extern "C"`. The two extern forms are not interchangeable — different calling conventions, safety models, and error surfaces (§11.4a). | §4.9, §11.4a |
| `E1108` | runtime revert | C | stable | `ChainError::InvalidTarget` — the target address passed to `chain::call` is not a contract, or has no function matching the declared selector. | §11.4a |
| `E1109` | runtime revert | C | stable | `ChainError::DecodingError` — the callee returned bytes that do not decode as the declared return type. Callee and caller disagree on the ABI. | §11.4a |
| `E1110` | compile error | C | reserved | Actor primitive used inside an `onchain` module. The `actor` keyword, the `spawn`, `send`, `send_timeout`, `select`, and `timeout(ms)` intrinsics are compile errors in `onchain` scope — on-chain execution is synchronous, single-threaded, and transactional, with no scheduler for any of these to run on. | §8.1, §11.1, §12.3 |
| `E1111` | compile error | C | reserved | Concurrency type used inside an `onchain` module. `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, and `JoinHandle<T>` are compile errors in `onchain` scope, whether in `storage` fields, event fields, function signatures, or local bindings. The forbid is on *spawning inside `onchain`*, not on depending on actor-using code through pure-function boundaries. | §8.1, §11.1, §12.3 |
| `E1112` | compile error | C | reserved | `@supervisor` or `@mailbox` attribute applied to an item declared inside an `onchain` module. Both attributes describe actor-runtime behavior and have no on-chain analog. | §11.1, §12.1, §12.3 |
| `E1113` | compile error | C | reserved | `async fn` declaration or `.await` operator used inside an `onchain` module. On-chain functions must be synchronous end-to-end within a transaction; there is no runtime scheduler for `.await` to suspend onto. | §11.1, §12.3 |
| `E1114` | — | C | vacant | Intentionally unassigned. Both directions of the `extern "C"` ↔ `extern onchain mod` symmetry violation are covered by `E1107`; per §18.4 frozen-on-publish, the slot stays vacant rather than being reassigned. | — |
| `E1115` | compile error | C | reserved | `Shared<T>` used inside an `onchain` module. Reference counting has no gas or storage meaning, and every on-chain value is scoped to the transaction frame, so no refcounted-sharing primitive is needed or well-defined on-chain. | §4.4a, §11.1, §12.3 |
| `E1116` | compile error | C | reserved | Floating-point math method called inside an `onchain` module. Every `f32`/`f64` method listed in §4.10 — classification, sign, rounding, min/max/clamp, power/root, exp/log, trig, hyperbolic, `mul_add`, angle conversion — is rejected. Transcendentals are not bit-reproducible across LLVM versions, platforms, and fast-math settings, and any drift would break consensus. `f32`/`f64` *values* may still be stored in fields, compared with `==`/`<`/`>`, and passed as arguments — only the §4.10 method calls are rejected. Use the integer math methods from §4.10 for all on-chain numeric work. | §4.10, §12.3 |
| `E1117` | compile error | C | reserved | `@fast_math` attribute applied to a function inside an `onchain` module. On-chain float determinism requires strict IEEE 754 semantics on every target; fast-math flags would allow bit-level drift across LLVM versions and break consensus. | §4.10, §12.1, §12.3 |
| `E1118` | compile error | C | reserved | `@overflow(wrapping)` applied to a function inside an `onchain` module. On-chain arithmetic is always checked — wrapping semantics would silently mask overflow bugs in financial code where precise reverts are mandatory. | §4.8, §11.1 |
| `E1119` | compile error | C | reserved | I/O-bound standard library module used inside an `onchain` module. `std::fs`, `std::net`, `std::io`, `std::db`, `std::web`, and `std::env` have no on-chain semantics — there is no filesystem, no network, no stdin/stdout, no database, no HTTP server, and no host-process environment in a transaction frame. | §11.1, §12.3 |
| `E1120` | compile error | C | reserved | `print` or `assert` general intrinsic called inside an `onchain` module. Both are off-chain-only intrinsics (§13.0). For on-chain failure paths, return `Err(...)` from the function so the transaction reverts deterministically with structured error data. | §11.1, §13.0 |
| `E1121` | compile error | C | reserved | `std::test` framework item used inside an `onchain` module. The `@test` attribute, `@property` attribute, the `assert_eq` / `assert_ne` / `assert_matches` test-only intrinsics, and the `TestFailure` / `Gen` / `Rng` library types are all rejected in `onchain` scope. On-chain code is tested off-chain by spawning a simulated execution context; the `@onchain_test` shape is deferred to a future amendment. Distinct from `E1410` / `E1411` (which catch the same items used outside any test context, on-chain or off). | §11.1, §12.3, §13.3 |
| `E1122` | compile error | C | reserved | Actor observability item used inside an `onchain` module. The `Handle<T>` introspection methods (`mailbox_len`, `mailbox_capacity`, `alive`, `actor_id`), the `std::actor::observe` module surface (`actor_info`, `actors`, `.by_supervisor`, `.by_name`), and the supervisor-rooted methods (`restart_count`, `restart_history`, `children`) are all rejected. Restated for completeness — `Handle<T>` itself is already an on-chain compile error per `E1111`, so all its methods are too; this code surfaces specifically when an observability call is the offending construct. | §8.12.7, §11.1, §12.3 |
| `E1123` | compile error | C | reserved | `ActorId` type used inside an `onchain` module. `ActorId` is an actor-runtime identifier with no on-chain analog. The spawn/death lifecycle that gives `ActorId` its monotonic-and-never-reused contract does not exist on-chain. | §8.12.5, §8.12.7 |

<!-- TODO: Promote E1110–E1113 and E1115–E1123 from `reserved` to `stable`
when the compiler emits the exact wording specified above. (E1114 is
intentionally vacant — see the row.) The on-chain prohibition list is
anchored in §11.1, §12.3, §8.1, §4.10, §4.4a, §8.12.7, and §13.3; new
prohibitions land here as the spec adds them. The next unfilled slot
is `E1124`. -->

## Cluster D — Actors / concurrency · `E1200–E1299`

Reserved range. Actor runtime and concurrency-primitive diagnostics will
earn codes in this range as they land.

| Code | Kind | Cluster | Status | Message / meaning | Spec ref |
|------|------|---------|--------|-------------------|----------|
| `E1210` | runtime error | D | reserved | `ObserveError::NotASupervisedChild` — `Handle<S>.restart_count(&child)` or `Handle<S>.restart_history(&child)` was called with a child handle that this supervisor does not actually supervise. Restart history is rooted on the spawning supervisor only. | §8.12.3 |
| `E1211` | compile error | D | reserved | `ActorId` comparison across runtime instances. v0.5.6 has one runtime per process; comparing `ActorId`s produced by different runtime instances is a compile error wherever the compiler can detect it. The multi-runtime story is deferred to a future amendment. | §8.12.5 |

<!-- TODO: Promote E1210 / E1211 from `reserved` to `stable` when the
runtime/compiler emits the exact wording specified above. -->

<!-- TODO: Populate the rest of cluster D once actor-runtime
diagnostics are implemented. -->

## Cluster E — FFI / extern · `E1300–E1399`

Reserved range. `extern "C"` and handler-safe FFI diagnostics will earn
codes in this range as they land.

<!-- TODO: Populate once FFI diagnostics are implemented. -->

## Cluster F — Attributes / derives / directives · `E1400–E1499`

Reserved range. Attribute and directive diagnostics (beyond the on-chain
ones already claimed in Cluster C) will earn codes in this range as they
land.

| Code | Kind | Cluster | Status | Message / meaning | Spec ref |
|------|------|---------|--------|-------------------|----------|
| `E1410` | compile error | F | reserved | Test-only intrinsic (`assert_eq`, `assert_ne`, `assert_matches`) used outside a `@test`-annotated function or `#[cfg(test)]` module. These intrinsics are removed by dead-code elimination in non-test builds and have no meaning in production code. | §13.0, §13.3.3 |
| `E1411` | compile error | F | reserved | Test-only prelude item (`TestFailure`, `Gen`, `Rng`, or one of the test assertion intrinsics) referenced outside a `@test`-annotated function or `#[cfg(test)]` module. The test-only prelude additions auto-import only under `#[cfg(test)]`; outside-test references prevent the test framework from leaking into release binaries. | §13.1, §13.3.8 |

<!-- TODO: Promote E1410 / E1411 from `reserved` to `stable` when the
compiler emits the exact wording specified above. -->

<!-- TODO: Populate the rest of cluster F once attribute-system
diagnostics are implemented. -->

## Warnings · `W0001–W0999`

Reserved range for non-error diagnostics. Warnings are emitted by the
compiler but do not fail the build.

<!-- TODO: Populate once the warning surface stabilizes. -->

## Lints · `L0001–L0999`

Reserved range for opt-in stylistic or best-practice diagnostics.
Intended to be a later-v0.5.x or v0.6.x addition.

<!-- TODO: Populate once a linter is specified. -->

## Internal compiler errors · `E9000+`

Reserved range for ICE codes (internal invariants violated in the
compiler). Not user-facing under normal operation. Every ICE carries a
code so diagnostic consumers can still pattern-match on it per §18.6 item
1.

<!-- TODO: Populate as compiler internals develop and specific ICE
paths are identified. -->

## Growth policy

1. **Registry-first.** A new code exists because a new row is added to
   this file. Spec sections reference codes by number; they do not create
   codes.
2. **Spec-anchored.** Every row must cite at least one spec section in
   the `Spec ref` column. A row without an anchoring spec section is a
   bug and should be removed or given a spec anchor.
3. **Frozen on publish.** Per §18.4, once a code ships in a released
   version, its `code → meaning` is immutable. Message text may evolve;
   semantics may not.
4. **Deprecate, don't reassign.** Retired codes are marked
   `status: deprecated` with a `superseded_by: <code>` pointer. Their
   numbers are never reassigned to new meanings.
