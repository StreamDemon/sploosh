# SPLOOSH Quick Reference — Web3 (v0.5.8) — LLM System Prompt Edition

Sploosh on-chain surface (§11). Native EVM and SVM (Solana SBF) targets.

> Assumes core language semantics from `LANGUAGE_SPEC_PROMPT_CORE.md`. Load both for on-chain work.

## Web3
```sploosh
onchain mod token {
    storage { balances: Map<Address, u256>, supply: u256 }
    pub fn transfer(to: Address, amt: u256) -> Result<(), TokenError> {
        let sender = ctx::caller();
        let bal = storage::get(&self.balances, sender)?;
        if bal < amt { return Err(TokenError::InsufficientBalance); }
        storage::set(&mut self.balances, sender, bal - amt);
        storage::set(&mut self.balances, to, storage::get(&self.balances, to)? + amt);
        emit Transfer { from: sender, to, amt };
        Ok(())
    }
}
```

## `ctx` API
- All targets: `ctx::caller() -> Address`, `ctx::self_address() -> Address`, `ctx::timestamp() -> u64`, `ctx::block_number() -> u64`.
- EVM-only: `ctx::value() -> u256` (requires `@payable`), `ctx::gas_remaining() -> u256`, `ctx::chain_id() -> u64`.
- SVM-only: `ctx::lamports() -> u64`, `ctx::program_id() -> Address`, `ctx::signer() -> Address`, `ctx::compute_units_remaining() -> u64`.

## Attributes
- `@payable` — function may receive `ctx::value()` (EVM); compile error without on the EVM target if `ctx::value()` is called.
- `@reentrant` — disables the per-contract reentrancy guard for that function only (see §11.3a).

## Storage Layout (§11.1a)
Target-pluggable; EVM adopts Solidity-compatible slots verbatim. Struct fields: sequential `u256` slots from 0 in declaration order, same-slot primitives right-aligned and packed (matches Solidity). `Map<K,V>` value at `keccak256(abi.encode(key, map_slot))` for value-type keys; nested maps recurse. `Vec<T>`: length at slot `s`, data at `keccak256(s)`. `String`: follows Solidity `bytes`/`string` (≤31-byte payloads inlined in slot `s`; longer payloads store data at `keccak256(s)`). `[T; N]` inline. SVM uses account-based storage; layout deferred to Solana amendment.

## Reentrancy Guard (§11.3a)
Runtime per-contract bool flag. Set on entry to any non-`@reentrant` `pub` on-chain function; cleared on return (Ok, Err, or revert). Cross-contract callback into a guarded function → `ChainError::Reentrancy` (revert). `@reentrant` disables check+set for that function only. Distinct from §8.10.1 actor `SelfCall` — same word, different layer.

## Cross-contract Calls (§11.4a)
Declare callee signatures with new syntax:
```sploosh
extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
}
let bal = chain::call(addr, token::balance_of, user)?;  // bal: u256; chain::call returns Result<T, ChainError> and `?` unwraps T
```
Sync on EVM (lowers to `CALL`). Solidity ABI for argument/return encoding. `?` propagates `ChainError::Reverted { data: Vec<u8> }` (revert data bounded by `RETURNDATACOPY`, allocated in caller's frame — same as Solidity). `ChainError = { Reverted, OutOfGas, Reentrancy, InvalidTarget, DecodingError }`. No delegatecall in v0.4.x. SVM: CPI lowering, concrete ABI deferred. **Distinct from `extern "C"` (§4.9)** — different calling convention, safety model, and error surface; not interchangeable.

## Gas / Compute Units (§11.7a)
Target-pluggable. **EVM**: gas; `ctx::gas_remaining() -> u256` EVM-only; `#[gas_limit(N)]` EVM-only advisory in ABI metadata (runtime OOG from VM, not annotation); costs per active hard fork's EIPs. **SVM**: compute units; `ctx::compute_units_remaining() -> u64` SVM-only; `#[gas_limit]` compile error on SVM. **Native/wasm**: all three are compile errors. **OOG → transaction revert**: all storage mutations and emitted events unwound; revert is transaction-wide and **unaffected by per-function attributes including `@reentrant`** — guard flag is unwound on revert, so failed calls cannot leave it stuck.

## Events `#[indexed]` (§11.5)
Up to 3 indexed fields per event variant on EVM (topics 1–3; topic 0 is signature hash). More than 3 on EVM → compile error. SVM accepts `#[indexed]` as a no-op.

## On-chain Prohibitions
Compile errors inside `onchain`: `actor`, `spawn`, `send`, `send_timeout`, `select`, `timeout(ms)`, `Handle<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, `JoinHandle<T>`, `@supervisor`, `@mailbox`, `async fn`/`.await`, `extern "C"`/`extern "C" async`, every `f32`/`f64` math method, `@fast_math`, `@overflow(wrapping)`, `Shared<T>`, `std::test` (incl. `assert_eq`/`assert_ne`/`assert_matches`), `std::actor::observe`, `ActorId`, `std::{fs,net,io,db,web,env}`. Float values, fields, comparisons, and integer math stay allowed.
