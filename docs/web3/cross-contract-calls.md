# Cross-Contract Calls

> Calling functions on other on-chain contracts.

## Basic Usage

```sploosh
let balance = chain::call(
    token_addr,
    token::balance_of,
    sender
)?;
```

## With Multiple Arguments

```sploosh
chain::call(
    token_addr,
    token::transfer_from,
    (sender, ctx::self_address(), amount)
)?;
```

## Error Handling

Cross-contract calls return `Result`. Always use `?` or explicit match to handle failures.

<!-- TODO: Expand with gas forwarding, call depth limits, and target-specific behavior -->
