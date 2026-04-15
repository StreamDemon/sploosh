# std::chain

> Cross-contract calls and on-chain utilities. Full call semantics in §11.4a.

**Available targets:** all (including onchain)

## Cross-Contract Calls

`chain::call` is the primary intrinsic in this module. Callee signatures must
be declared at module level via `extern onchain mod` (§11.4a):

```sploosh
use std::chain;

extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError>;
}

// Single-argument call:
let bal = chain::call(token_addr, token::balance_of, sender)?;

// Multi-argument call (tuple):
chain::call(token_addr, token::transfer, (recipient, amount))?;
```

### Signature

```sploosh
chain::call<Args, T, E>(target: Address, fn: ExternFn<Args, T, E>, args: Args)
    -> Result<T, ChainError>
```

### ChainError

```sploosh
@error
pub enum ChainError {
    Reverted { data: Vec<u8> },  // callee reverted; `data` is the revert payload
    OutOfGas,                     // callee exhausted forwarded gas
    Reentrancy,                   // callee hit its §11.3a guard
    InvalidTarget,                // address is not a contract / selector mismatch
    DecodingError,                // callee return bytes do not decode as T
}
```

`?` on a `chain::call` result propagates `ChainError` to the caller's
surrounding `Result`. The callee's own domain error (e.g., `TokenError`) is
carried inside `ChainError::Reverted { data }` and can be decoded via the
`@error`-generated decoder on the callee's enum.

## Target Semantics

- **EVM.** `chain::call` lowers to `CALL`. Synchronous. Solidity ABI
  encoding with a 4-byte function selector derived from
  `keccak256(signature_string)[0..4]`. Gas forwarded per EVM default
  (remaining minus 1/64, EIP-150). No delegatecall in v0.4.x.
- **SVM.** `chain::call` lowers to a Solana CPI instruction. User-level
  surface (synchronous-looking `Result<T, ChainError>`, `?` propagation) is
  preserved; concrete CPI ABI and account-passing conventions are deferred
  to the Solana-targeting amendment.
- **native / wasm.** Not meaningful off-chain; only the declaration surface
  (`extern onchain mod`) is available for use by off-chain code that will
  also compile for on-chain.

## Compiler Intrinsic, Not a Library Function

`chain::call` is a compiler intrinsic — its behavior is baked into the
compiler rather than implemented as ordinary Sploosh code. It cannot be
imported under a different name, passed as a function pointer, or
re-exported.

<!-- TODO: When the EVM / SVM backends land, document address construction,
ABI encoding/decoding helpers, and any SVM-specific account manifest surface
that sits alongside chain::call. -->

## See Also

- §11.4a — Cross-Contract ABI and Call Semantics
- [docs/web3/cross-contract-calls.md](../web3/cross-contract-calls.md)
- [docs/web3/payable-and-reentrancy.md](../web3/payable-and-reentrancy.md) — how the guard interacts with callbacks
