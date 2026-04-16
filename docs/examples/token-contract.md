# Example: Token Contract

> An ERC-20 style token implemented as an on-chain module.

## contracts/token.sp

```sploosh
@error
enum TokenError {
    InsufficientBalance,
    Unauthorized,
}

onchain mod token {
    // Storage layout on EVM: sequential slots from 0 in declaration order
    // (§11.1a). `balances` occupies slot 0 as a Map header; its entries
    // live at keccak256(abi.encode(key, 0)). `total_supply` is slot 1.
    // `owner` is slot 2 (left-padded with 12 zero high bytes; the low-order
    // 20 bytes hold the address, per §3.1). Layout matches Solidity
    // verbatim, so a Solidity indexer or proxy can read this storage.
    storage {
        balances: Map<Address, u256>,
        total_supply: u256,
        owner: Address,
    }

    // Guarded by the §11.3a per-contract reentrancy flag by default —
    // no explicit nonReentrant modifier needed.
    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError> {
        let sender = ctx::caller();
        let bal = storage::get(&self.balances, sender)?;
        if bal < amount {
            return Err(TokenError::InsufficientBalance);
        }
        storage::set(&mut self.balances, sender, bal - amount);
        storage::set(&mut self.balances, to, storage::get(&self.balances, to)? + amount);
        emit Transfer { from: sender, to, amount };
        Ok(())
    }

    pub fn balance_of(account: Address) -> Result<u256, TokenError> {
        Ok(storage::get(&self.balances, account).unwrap_or(0))
    }
}

onchain enum Event {
    // #[indexed] marks topic slots on EVM — up to 3 per variant
    // (topic 0 is the event signature hash). §11.5.
    Transfer {
        #[indexed] from: Address,
        #[indexed] to: Address,
        amount: u256,
    },
}
```

## Key Patterns

- **`@error`** auto-generates `Display` and `Error` impls
- **`storage` block** defines persistent on-chain state
- **`ctx::caller()`** replaces Solidity's `msg.sender`
- **`emit`** for on-chain event logging
- **Non-reentrant by default** -- per-contract runtime flag (§11.3a); no explicit guard needed. `@reentrant` opts out per-function
- **Solidity-compatible storage** -- the `storage` block lays out to Solidity-compatible EVM slots (§11.1a), enabling interop with Solidity proxies and indexers
- **`#[indexed]` event fields** -- up to 3 per variant on EVM, mirrors Solidity's indexed event parameters (§11.5)
- **Result-based errors** -- no `require`/`revert`, just return `Err`
