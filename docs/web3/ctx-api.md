# The ctx Module (On-Chain Context)

> Blockchain execution context available inside `onchain` modules.

## Universal Functions (All Targets)

| Function | Return Type | Purpose |
|----------|-------------|---------|
| `ctx::caller()` | `Address` | Address that invoked this function |
| `ctx::self_address()` | `Address` | This contract's address |
| `ctx::timestamp()` | `u256` | Current block timestamp (seconds) |
| `ctx::block_number()` | `u256` | Current block number / slot |

## EVM-Specific

| Function | Return Type | Purpose |
|----------|-------------|---------|
| `ctx::value()` | `u256` | ETH sent with the call (wei). Requires `@payable` |
| `ctx::gas_remaining()` | `u256` | Remaining gas |
| `ctx::chain_id()` | `u256` | EVM chain ID |

## SVM-Specific (Solana)

| Function | Return Type | Purpose |
|----------|-------------|---------|
| `ctx::lamports()` | `u64` | SOL sent with the instruction |
| `ctx::program_id()` | `Address` | This program's address |
| `ctx::signer()` | `Address` | Transaction signer |

## Usage

```sploosh
onchain mod vault {
    pub fn withdraw(amount: u256) -> Result<(), VaultError> {
        let sender = ctx::caller();
        let balance = storage::get(&self.balances, sender)?;
        if balance < amount {
            return Err(VaultError::InsufficientBalance);
        }
        // ...
    }
}
```

Calling `ctx::value()` in a non-`@payable` function is a compile-time error.

## Implementation Note

All `ctx::*` functions, `storage::get`/`storage::set`, `chain::call`, and `emit` are **compiler intrinsics**, not regular library functions. They are only available inside `onchain` modules -- using them outside `onchain` is a compile-time error.
