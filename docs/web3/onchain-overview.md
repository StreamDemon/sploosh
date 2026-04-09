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

**Available inside onchain:** `std::math`, `std::crypto`, `std::chain`, `std::collections`, and all core types.

## Key Differences from Regular Modules

- All state lives in `storage` blocks (persistent on-chain)
- Functions are non-reentrant by default
- No references in public function parameters (messages cross transaction boundaries)
- Events via `emit` keyword
- Blockchain context via `ctx` module

<!-- TODO: Expand with compilation model details once EVM/SVM backends are implemented -->
