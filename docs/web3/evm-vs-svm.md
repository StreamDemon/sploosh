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

<!-- TODO: Expand with account model differences, storage model differences, and migration patterns -->
