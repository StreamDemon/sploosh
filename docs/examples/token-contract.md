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
    storage {
        balances: Map<Address, u256>,
        total_supply: u256,
        owner: Address,
    }

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
    Transfer { from: Address, to: Address, amount: u256 },
}
```

## Key Patterns

- **`@error`** auto-generates `Display` and `Error` impls
- **`storage` block** defines persistent on-chain state
- **`ctx::caller()`** replaces Solidity's `msg.sender`
- **`emit`** for on-chain event logging
- **Non-reentrant by default** -- no reentrancy guard needed
- **Result-based errors** -- no `require`/`revert`, just return `Err`
