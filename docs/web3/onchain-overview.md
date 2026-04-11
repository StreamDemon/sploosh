# On-Chain Overview

> How `onchain` modules work, what makes them different, and stdlib restrictions.

## The onchain Keyword

`onchain` modules contain code that compiles to blockchain bytecode (EVM or SVM). They have special constraints enforced at compile time.

```sploosh
onchain mod token {
    storage {
        balances: Map<Address, u256>,
        total_supply: u256,
    }

    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError> {
        let sender = ctx::caller();
        // ...
    }
}
```

## Standard Library Restrictions

The following modules are compile-time errors inside `onchain`:

- `std::fs` -- no filesystem
- `std::net` -- no networking
- `std::io` -- no stdin/stdout
- `std::db` -- no database
- `std::web` -- no HTTP server
- `std::env` -- no environment variables

**Concurrency primitives are also forbidden on-chain.** The `actor` keyword, the `spawn`, `send`, `send_timeout`, `select`, and `timeout(ms)` intrinsics, the `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, and `JoinHandle<T>` types, and the `@supervisor` and `@mailbox` attributes are all compile errors inside `onchain`. `extern "C"` and `extern "C" async` FFI blocks are also rejected. On-chain execution is synchronous, single-threaded, and transactional — there is no runtime scheduler for any of these to run on. Transitive imports of native modules that internally use actors are still allowed, provided the functions called across the `onchain` boundary do not themselves touch actor intrinsics. See §8.1, §11.1, and §12.3 of the language specification for the full list.

**Available inside onchain:** `std::math` (integer methods only — floating-point methods are forbidden), `std::crypto`, `std::chain`, `std::collections`, and all core types.

## Key Differences from Regular Modules

- All state lives in `storage` blocks (persistent on-chain)
- Functions are non-reentrant by default
- No references in public function parameters (messages cross transaction boundaries)
- Events via `emit` keyword
- Blockchain context via `ctx` module
- **No concurrency** -- on-chain execution is sequential within a transaction. No actors, no async, no channels, no FFI. On-chain functions are non-reentrant by default; the `@reentrant` attribute is the explicit opt-in (§11.3). This is a wholly separate mechanism from the off-chain actor-handler `SelfCall` detection — the two do not overlap.

## Checked Arithmetic on Chain

All integer arithmetic inside `onchain` modules is **always checked**, regardless of build mode (debug or release). An overflow or underflow will cause the transaction to **revert**.

- `@overflow(wrapping)` is a **compile error** inside `onchain` modules -- wrapping arithmetic is never permitted on chain.
- `u256` arithmetic is always checked, just like every other integer type.

This ensures deterministic, safe behavior for all on-chain financial calculations.

## Compiler Intrinsics

The following on-chain APIs are **compiler intrinsics**, not regular function calls:

- `ctx::*` -- blockchain context (caller, value, block info, etc.)
- `storage::*` -- persistent storage reads and writes
- `chain::call` -- cross-contract calls
- `emit` -- event emission

The compiler lowers these directly to the target bytecode (EVM/SVM). They cannot be imported, re-exported, or passed as function pointers.

<!-- TODO: Expand with compilation model details once EVM/SVM backends are implemented -->
