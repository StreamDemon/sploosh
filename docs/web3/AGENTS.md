# AGENTS.md — `docs/web3/`

On-chain semantics: storage, events, ctx API, cross-contract calls, EVM/SVM target differences. Mirrors spec §11 and §12 attribute interactions.

## Identity

Sploosh's web3 surface. These pages cover what `onchain mod` modules can do, what they can't, and how they lower to EVM (Solidity-compatible) and SVM (Solana SBF) targets.

## Files

```
onchain-overview.md      storage-and-state.md       events.md
ctx-api.md               cross-contract-calls.md
payable-and-reentrancy.md
evm-vs-svm.md            deploying-contracts.md
```

## Patterns & Conventions

- **State the prohibition, then the rule.** On-chain is restrictive: list what's forbidden first, what's allowed second. Compile-time errors are first-class — name them.
- **EVM is the reference target.** When EVM and SVM differ, lead with EVM and call out SVM divergences explicitly.
- **`#[indexed]` events** (§11.5): max 3 indexed fields per variant on EVM (topics 1–3); SVM treats `#[indexed]` as no-op.
- **Reentrancy guard** (§11.3a) is **distinct from** actor `SelfCall` (§8.10.1). Same word, different layer. Don't conflate.
- **Cross-contract calls** (§11.4a) use `extern onchain mod` + `chain::call(addr, fn, args)?`. Distinct from `extern "C"` (§4.9) — different calling convention, error surface, ABI.

## Touch Points

- `storage-and-state.md` — Solidity-compatible slot layout (§11.1a). Maps hash to `keccak256(abi.encode(key, slot))`. Vec/String length-then-data convention.
- `payable-and-reentrancy.md` — `@payable` (must annotate to receive ETH/lamports), `@reentrant` (opts out of guard for that fn only).
- `evm-vs-svm.md` — gas vs compute units, `ctx::gas_remaining()` (EVM) vs `ctx::compute_units_remaining()` (SVM), `#[gas_limit(N)]` (EVM-only).
- `cross-contract-calls.md` — `ChainError = { Reverted, OutOfGas, Reentrancy, InvalidTarget, DecodingError }`. No delegatecall in v0.4.x.

## JIT Index Hints

```pwsh
# Find every on-chain prohibition
rg -n "compile error|forbidden|not available on-chain" docs/web3

# Find ctx API references
rg -n "ctx::\w+" docs

# Find every event/emit example
rg -n "emit \w+" docs

# Confirm SVM caveats are up to date
rg -n "Solana|SVM|compute units" docs/web3
```

## Common Gotchas

- **Floats are not banned on-chain — float *methods* are.** `f64` fields, `==`, `<`, function args still work; only `.sqrt()`, `.abs()`, `.sin()`, etc. fail to compile.
- **OOG (out-of-gas) revert is transaction-wide.** Storage mutations and emitted events are unwound, **including the reentrancy guard flag** (§11.7a). `@reentrant` does not change this.
- **`@payable` is required to read `ctx::value()` on EVM.** Missing the attribute is a compile error.
- **`#[gas_limit(N)]` is EVM-only and advisory** — runtime OOG comes from the VM, not the annotation. SVM use is a compile error.
- **Layout for SVM is deferred** to a Solana amendment. Don't invent slot mechanics for SVM that the spec doesn't endorse.

## Pre-PR Checks

```pwsh
# Web3 changes should usually touch the spec too
git diff --name-only main...HEAD | findstr -r "web3 spec-plans"

# Cross-check stdlib chain.md when touching cross-contract calls
git diff --name-only main...HEAD | findstr -r "chain.md cross-contract"
```
