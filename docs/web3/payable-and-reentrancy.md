# Payable Functions and Reentrancy

> Receiving native tokens and reentrancy protection.

## @payable

Functions that receive native tokens must be annotated with `@payable`:

```sploosh
@payable
pub fn deposit() -> Result<(), VaultError> {
    let sender = ctx::caller();
    let amount = ctx::value();
    // ...
}
```

Calling `ctx::value()` without `@payable` is a compile-time error.

## Non-Reentrant by Default

All on-chain functions are non-reentrant by default. A function cannot be called again while it is already executing. This prevents the most common class of smart contract vulnerabilities.

To opt in to reentrancy (discouraged):

```sploosh
@reentrant
pub fn risky_function() -> Result<(), Error> { /* ... */ }
```

<!-- TODO: Add examples of reentrancy attacks and why the default protection matters -->
