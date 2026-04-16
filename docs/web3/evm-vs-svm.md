# EVM vs SVM

> Target differences, portability, and conditional compilation.

## Target-Specific Code

Use compiler directives to write target-specific implementations:

```sploosh
#[target(evm)]
pub fn get_sent_value() -> u256 {
    ctx::value()
}

#[target(svm)]
pub fn get_sent_value() -> u64 {
    ctx::lamports()
}
```

## Key Differences

| Aspect | EVM (Ethereum) | SVM (Solana) |
|--------|---------------|--------------|
| Native token context | `ctx::value()` (u256, wei) | `ctx::lamports()` (u64) |
| Caller | `ctx::caller()` | `ctx::signer()` |
| Contract address | `ctx::self_address()` | `ctx::program_id()` |
| Gas model | Gas-based | Compute units |
| Integer size | u256 native | u64 native |

## Portable Code

Code that uses only universal `ctx` functions and standard types works on both targets:

```sploosh
pub fn get_caller() -> Address {
    ctx::caller()       // works on both EVM and SVM
}
```

Use `#[cfg(target = "evm")]` and `#[cfg(target = "svm")]` for conditional compilation.

## Storage Model Differences

Sploosh specifies storage as a **target-pluggable abstraction** (§11.1a).
The source-level surface — `storage { ... }` declarations, `storage::get` /
`storage::set`, `Map<K, V>` field type — is identical on both targets. The
concrete bytes on chain differ:

| Aspect | EVM | SVM |
|--------|-----|-----|
| Persistence unit | 32-byte slots in a per-contract storage root | Accounts owned by the program |
| Struct field layout | Sequential slots from 0 in declaration order, Solidity packing within slots | Program-defined account schema (Sploosh-defined in a future amendment) |
| `Map<K, V>` entry | Value at `keccak256(abi.encode(key, map_slot))` | Target-divergent; deferred to Solana amendment |
| Nested maps | Recurse via repeated `keccak256` derivation | Deferred |
| Dynamic (`Vec<T>`, `String`) | Length at slot `s`, data at `keccak256(s)` | Deferred |
| Fixed `[T; N]` | Inline sequential slots, no hashing | Deferred |
| Interop | Solidity-compatible storage layout; can coexist with Solidity contracts, proxies, indexers | Solana account schema; Sploosh-defined once the SVM amendment lands |

See [storage-and-state.md](./storage-and-state.md) for the full EVM
reference realization.

## Gas Model Differences

On-chain execution is metered. Sploosh exposes this via target-specific
intrinsics and an advisory directive:

| Aspect | EVM | SVM |
|--------|-----|-----|
| Metering unit | Gas | Compute units |
| Remaining-unit intrinsic | `ctx::gas_remaining() -> u256` | `ctx::compute_units_remaining() -> u64` |
| `#[gas_limit(N)]` | Advisory; surfaced in deployed ABI metadata | Compile error |
| Cost source | EVM Yellow Paper + active hard fork EIPs (2929 warm/cold, 3529 refunds, ...) | Solana runtime compute budget (runtime-version-dependent) |
| OOG effect | Transaction revert: all storage mutations and emitted events unwound | Instruction abort: state not committed |
| Native / wasm | All gas intrinsics are compile errors — gas is on-chain-only | Same |

**OOG unwind is transaction-wide and unaffected by per-function attributes**
including `@reentrant` — a failed call cannot leave the reentrancy flag
stuck set (see [payable-and-reentrancy.md](./payable-and-reentrancy.md)).
See §11.7a for the full gas-model specification.

## See Also

- §11.1a — Storage Layout specification
- §11.7a — Gas Model specification
- [storage-and-state.md](./storage-and-state.md) — persistent storage model
- [ctx-api.md](./ctx-api.md) — context intrinsics reference
