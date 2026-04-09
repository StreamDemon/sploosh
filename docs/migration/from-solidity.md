# Coming from Solidity

> How Sploosh's on-chain model maps to Solidity concepts.

## Mapping Concepts

| Solidity | Sploosh | Notes |
|----------|---------|-------|
| `contract MyToken { }` | `onchain mod my_token { }` | Module-based, not class-based |
| `mapping(address => uint256)` | `storage { balances: Map<Address, u256> }` | Explicit storage block |
| `msg.sender` | `ctx::caller()` | Function call, not magic global |
| `msg.value` | `ctx::value()` | Requires `@payable` annotation |
| `block.timestamp` | `ctx::timestamp()` | Same semantics |
| `require(cond, "msg")` | `if !cond { return Err(...); }` | Errors are values |
| `revert("msg")` | `return Err(...)` | Explicit error return |
| `event Transfer(...)` | `emit Transfer { ... }` | Struct-like event syntax |
| `modifier nonReentrant` | Non-reentrant by default | Opt-in with `@reentrant` |
| `external`/`public`/`internal`/`private` | `pub` or private | Two levels only |
| `payable` | `@payable` | Attribute syntax |
| `interface` | `trait` | Same concept |

## Key Advantages Over Solidity
- **Type safety** -- no implicit conversions, no integer overflow in safe code
- **Result-based errors** -- compiler forces you to handle every failure
- **Non-reentrant by default** -- most common vulnerability class eliminated
- **Same language off-chain** -- write backend + contract in one language
- **Pattern matching** -- exhaustive match replaces chains of if/else

## Example Comparison

```solidity
// Solidity
function transfer(address to, uint256 amount) external {
    require(balances[msg.sender] >= amount, "insufficient");
    balances[msg.sender] -= amount;
    balances[to] += amount;
    emit Transfer(msg.sender, to, amount);
}
```

```sploosh
// Sploosh
pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError> {
    let sender = ctx::caller();
    let bal = storage::get(&self.balances, sender)?;
    if bal < amount { return Err(TokenError::InsufficientBalance); }
    storage::set(&mut self.balances, sender, bal - amount);
    storage::set(&mut self.balances, to, storage::get(&self.balances, to)? + amount);
    emit Transfer { from: sender, to, amount };
    Ok(())
}
```
