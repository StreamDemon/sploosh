# Compiler Errors

> Stable registry of Sploosh diagnostic codes. The **format** of a diagnostic
> (record shape, severity levels, applicability vocabulary, output modes,
> stability contract) is specified in `docs/spec-plans/LANGUAGE_SPEC.md` §18.
> This file is the **registry** — the canonical `code → meaning` mapping
> that §18.4 freezes.

Each diagnostic that the compiler or runtime emits carries a stable code of
the form `E<NNNN>` (error), `W<NNNN>` (warning), or `L<NNNN>` (lint). The
`Kind` column distinguishes **compile errors** (raised at build time, no
deployed artifact) from **runtime reverts** (raised during transaction
execution). Runtime-revert entries mirror variants defined in their spec
sections (§8.7, §11.3a, §11.4a, §11.7a); the variant semantics are
specified there, and this page is the canonical source of each stable code
number. The `Cluster` column locates the code in the partitioning defined
in §18.2. The `Status` column is one of `stable` (published code — the
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

## Cluster D — Actors / concurrency · `E1200–E1299`

Reserved range. Actor runtime and concurrency-primitive diagnostics will
earn codes in this range as they land.

<!-- TODO: Populate once actor-runtime diagnostics are implemented. -->

## Cluster E — FFI / extern · `E1300–E1399`

Reserved range. `extern "C"` and handler-safe FFI diagnostics will earn
codes in this range as they land.

<!-- TODO: Populate once FFI diagnostics are implemented. -->

## Cluster F — Attributes / derives / directives · `E1400–E1499`

Reserved range. Attribute and directive diagnostics (beyond the on-chain
ones already claimed in Cluster C) will earn codes in this range as they
land.

<!-- TODO: Populate once attribute-system diagnostics are implemented. -->

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
