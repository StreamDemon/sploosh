# Compiler Errors

> Stable registry of Sploosh diagnostic codes. The **format** of a diagnostic
> (fields, stability contract, JSON mode, suggested-fix applicability) is
> specified in `docs/spec-plans/LANGUAGE_SPEC.md` §18. This page is the
> canonical **registry** of code numbers keyed by cluster; it holds the
> `code → meaning` mapping that §18.4 freezes.

Each diagnostic that the compiler or runtime emits carries a stable code of
the form `E<NNNN>` (error), `W<NNNN>` (warning), or `L<NNNN>` (lint). The
`Kind` column distinguishes **compile errors** (raised at build time, no
deployed artifact) from **runtime reverts** (raised during transaction
execution); runtime-revert entries mirror variants defined in their spec
sections (§8.7, §11.3a, §11.4a, §11.7a), and the variant semantics are
specified there. The `Cluster` column locates the code in the partitioning
defined in §18.2. The `Status` column is one of `stable` (published code
whose meaning is frozen per §18.4), `provisional` (defined in this revision
but subject to change before the next release), or `deprecated` (kept for
back-compat; see `Superseded by` in the row-level notes).

## Code clusters (§18.2)

| Range         | Cluster | Topic                                                           |
|---------------|---------|-----------------------------------------------------------------|
| `E0001–E0999` | A       | Lexical / parser / basic syntax                                 |
| `E1000–E1099` | B       | Type system, trait coherence, ownership, lifetimes              |
| `E1100–E1199` | C       | On-chain                                                        |
| `E1200–E1299` | D       | Actors / concurrency                                            |
| `E1300–E1399` | E       | FFI / extern                                                    |
| `E1400–E1499` | F       | Attributes / derives / directives                               |
| `W0001–W0999` | —       | Warnings                                                        |
| `L0001–L0999` | —       | Lints                                                           |
| `E9000+`      | —       | Internal compiler errors (ICE). Reserved; not user-facing.      |

New codes land inside their cluster's range, never reuse a retired number,
and always reference the spec section that defines the variant (see the
**Growth policy** section below).

## Row schema

| Column          | Purpose                                                                   |
|-----------------|---------------------------------------------------------------------------|
| `Code`          | `E<NNNN>` / `W<NNNN>` / `L<NNNN>`. Stable identifier.                     |
| `Kind`          | `compile error`, `runtime revert`, `warning`, or `lint`.                  |
| `Cluster`       | `A`–`F` letter matching the range table above.                            |
| `Status`        | `stable` / `provisional` / `deprecated`.                                  |
| `Message / meaning` | One-sentence summary; §18.1 `message` field renders from here.        |
| `Spec ref`      | Section of `LANGUAGE_SPEC.md` defining the variant semantics.             |

## On-chain (Cluster C, v0.4.4) — E1101–E1199

| Code    | Kind            | Cluster | Status | Message / meaning | Spec ref |
|---------|-----------------|---------|--------|-------------------|----------|
| `E1101` | runtime revert  | C       | stable | `ChainError::Reentrancy` — a cross-contract callback re-entered a non-`@reentrant` `pub` function of a contract whose reentrancy flag was already set. The transaction reverts and all state mutations are unwound. | §11.3a |
| `E1102` | runtime revert  | C       | stable | `ChainError::OutOfGas` — the callee of a `chain::call` exhausted its forwarded gas. The callee's frame reverts; the caller observes `Err(ChainError::OutOfGas)` provided it retains enough gas to handle the error path. Transaction-wide OOG reverts the entire transaction. | §11.4a, §11.7a |
| `E1103` | compile error   | C       | stable | Gas intrinsic used on the wrong target. `ctx::gas_remaining()` is EVM-only; `ctx::compute_units_remaining()` is SVM-only; `#[gas_limit(N)]` is EVM-only. Using any of these on an unsupported target is a compile error. | §11.7a |
| `E1104` | compile error   | C       | stable | Invalid `extern onchain mod` interface. Common causes: function body present (only signatures allowed), signature missing return type `Result<T, E>`, error type not in scope, block nested inside `extern "C"` or inside a function body. | §11.4a |
| `E1105` | compile error   | C       | stable | `#[indexed]` used outside an event variant field, or more than three `#[indexed]` fields on a single event variant compiled for EVM (topic slots 1–3 are the only indexed slots; topic 0 is reserved for the event signature hash). | §11.5 |
| `E1106` | compile error   | C       | stable | `@reentrant` applied to a non-`pub` or non-on-chain function. The attribute only disables the §11.3a per-contract guard; it has no meaning outside `pub` functions of an `onchain mod`. | §11.3a, §12.1 |
| `E1107` | compile error   | C       | stable | `extern "C"` block declared inside an `onchain` module, or `extern onchain mod` declared inside `extern "C"`. The two extern forms are not interchangeable — different calling conventions, safety models, and error surfaces (§11.4a). | §4.9, §11.4a |
| `E1108` | runtime revert  | C       | stable | `ChainError::InvalidTarget` — the target address passed to `chain::call` is not a contract, or has no function matching the declared selector. | §11.4a |
| `E1109` | runtime revert  | C       | stable | `ChainError::DecodingError` — the callee returned bytes that do not decode as the declared return type. Callee and caller disagree on the ABI. | §11.4a |

## Growth policy

1. **Assign in range.** New codes must fall inside the cluster range declared
   in §18.2 above. Running out of numbers in a cluster is itself a spec
   question — reserve a new range before crowding.
2. **Cite the spec.** Every row must reference the spec section(s) that
   define the variant's semantics. If no such section exists, the spec work
   lands first.
3. **Freeze on release.** Once a code ships in a released version, its
   `code → meaning` mapping is frozen (§18.4). Retirement uses the
   `deprecated` status with a `Superseded by` pointer in a row-level note,
   not number reuse.
4. **Catalog-first.** New diagnostics added to the compiler must land
   simultaneously in this file and in the spec section that describes the
   variant. A compiler PR that emits a code not listed here is rejected.
