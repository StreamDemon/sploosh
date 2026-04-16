# Compiler Errors

> Catalog of compiler error codes with explanations and suggested fixes.

This page is the stable registry of Sploosh diagnostic codes, covering
**both** compile-time errors emitted by the compiler **and** runtime reverts
emitted by the VM / actor runtime (e.g., variants of `ChainError` and
`ActorError`). Codes follow the pattern `E<NNNN>` and are assigned in
blocks by topic; the full catalog will grow as the compiler lands. The
`Kind` column distinguishes compile errors (raised at build time, no
deployed artifact) from runtime reverts (raised during transaction
execution). Runtime-revert entries mirror variants defined in their spec
sections (§8.7, §11.3a, §11.4a, §11.7a); the variant semantics are
specified there, but this page is the canonical source of its stable code
number.

## On-chain (Cluster C, v0.4.4) — E1101–E1199

| Code | Kind | Message / meaning | Spec ref |
|------|------|-------------------|----------|
| `E1101` | runtime revert | `ChainError::Reentrancy` — a cross-contract callback re-entered a non-`@reentrant` `pub` function of a contract whose reentrancy flag was already set. The transaction reverts and all state mutations are unwound. | §11.3a |
| `E1102` | runtime revert | `ChainError::OutOfGas` — the callee of a `chain::call` exhausted its forwarded gas. The callee's frame reverts; the caller observes `Err(ChainError::OutOfGas)` provided it retains enough gas to handle the error path. Transaction-wide OOG reverts the entire transaction. | §11.4a, §11.7a |
| `E1103` | compile error | Gas intrinsic used on the wrong target. `ctx::gas_remaining()` is EVM-only; `ctx::compute_units_remaining()` is SVM-only; `#[gas_limit(N)]` is EVM-only. Using any of these on an unsupported target is a compile error. | §11.7a |
| `E1104` | compile error | Invalid `extern onchain mod` interface. Common causes: function body present (only signatures allowed), signature missing return type `Result<T, E>`, error type not in scope, block nested inside `extern "C"` or inside a function body. | §11.4a |
| `E1105` | compile error | `#[indexed]` used outside an event variant field, or more than three `#[indexed]` fields on a single event variant compiled for EVM (topic slots 1–3 are the only indexed slots; topic 0 is reserved for the event signature hash). | §11.5 |
| `E1106` | compile error | `@reentrant` applied to a non-`pub` or non-on-chain function. The attribute only disables the §11.3a per-contract guard; it has no meaning outside `pub` functions of an `onchain mod`. | §11.3a, §12.1 |
| `E1107` | compile error | `extern "C"` block declared inside an `onchain` module, or `extern onchain mod` declared inside `extern "C"`. The two extern forms are not interchangeable — different calling conventions, safety models, and error surfaces (§11.4a). | §4.9, §11.4a |
| `E1108` | runtime revert | `ChainError::InvalidTarget` — the target address passed to `chain::call` is not a contract, or has no function matching the declared selector. | §11.4a |
| `E1109` | runtime revert | `ChainError::DecodingError` — the callee returned bytes that do not decode as the declared return type. Callee and caller disagree on the ABI. | §11.4a |

<!-- TODO: Expand this catalog as the compiler is implemented. Each compile
error should eventually include: error code, error message, explanation,
example code that triggers the error, and how to fix it. -->
