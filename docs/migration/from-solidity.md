# Coming from Solidity

> How Sploosh's on-chain model maps to Solidity concepts.

## Mapping Concepts

| Solidity | Sploosh | Notes |
|----------|---------|-------|
| `contract MyToken { }` | `onchain mod my_token { }` | Module-based, not class-based |
| `mapping(address => uint256)` | `storage { balances: Map<Address, u256> }` | Storage layout is Solidity-compatible on EVM — see §11.1a and "Storage Layout" below |
| `mapping(address => mapping(uint256 => ...))` | `Map<Address, Map<u256, ...>>` | Nested maps recurse via `keccak256(key ++ slot)` on EVM, identical to Solidity |
| `msg.sender` | `ctx::caller()` | Function call, not magic global |
| `msg.value` | `ctx::value()` | Requires `@payable` annotation |
| `block.timestamp` | `ctx::timestamp()` | Same semantics |
| `gasleft()` | `ctx::gas_remaining()` | EVM-only; compile error on other targets |
| `require(cond, "msg")` | `if !cond { return Err(...); }` | Errors are values |
| `revert("msg")` | `return Err(...)` | Explicit error return; caller sees `ChainError::Reverted { data }` |
| `event Transfer(address indexed from, ..., uint256 amount)` | `Transfer { #[indexed] from: Address, ..., amount: u256 }` | `#[indexed]` field marker, up to 3 per variant on EVM (§11.5) |
| `modifier nonReentrant` | Non-reentrant by default | Per-contract runtime flag (§11.3a); opt-out with `@reentrant` |
| `external`/`public`/`internal`/`private` | `pub` or private | Two levels only |
| `payable` | `@payable` | Attribute syntax |
| `interface IToken { function ...; }` | `extern onchain mod token { pub fn ...; }` | Cross-contract interface declaration (§11.4a) |
| `token.balanceOf(x)` | `chain::call(addr, token::balance_of, x)?` | Explicit address + function reference; `?` propagates `ChainError::Reverted` |
| Solidity `interface` (type-level) | `trait` | Only when the type is used as a generic constraint off-chain; cross-contract calls use `extern onchain mod` |

## Key Advantages Over Solidity
- **Type safety** -- no implicit conversions, no integer overflow in safe code
- **Result-based errors** -- compiler forces you to handle every failure
- **Non-reentrant by default** -- most common vulnerability class eliminated. Implemented as a per-contract runtime flag checked on entry to every non-`@reentrant` `pub` function (§11.3a); the guard is unwound on revert, so failed calls cannot leave it stuck set.
- **Same language off-chain** -- write backend + contract in one language
- **Pattern matching** -- exhaustive match replaces chains of if/else

## Storage Layout

On EVM, Sploosh adopts the **Solidity storage layout verbatim** so that
Sploosh contracts interoperate with existing Solidity contracts, proxies,
and indexers without a translation layer:

- Struct fields occupy sequential 32-byte slots from `0` in declaration order.
- Same-slot primitives are right-aligned and packed per Solidity rules.
- `Map<K, V>` entries live at `keccak256(abi.encode(key, map_slot))`.
- Nested maps recurse identically.
- `Vec<T>` stores length at slot `s`, data at `keccak256(s)`; `String`
  follows Solidity `bytes` / `string` rules (payloads ≤ 31 bytes are inlined
  in slot `s`; longer payloads store data at `keccak256(s)`).
- `[T; N]` occupies inline sequential slots, no hashing.

A Sploosh contract can be deployed behind an existing Solidity upgrade proxy
or read by existing Solidity storage-layout tooling (e.g., `forge inspect`)
provided the Sploosh `storage { ... }` fields are declared in the same
order as the Solidity contract's state variables. See §11.1a and
[docs/web3/storage-and-state.md](../web3/storage-and-state.md) for the full
layout specification.

SVM uses an account-based model and does not derive slots via `keccak256`.
The Sploosh surface is identical across targets; the concrete SVM layout is
deferred to the Solana-targeting amendment.

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
