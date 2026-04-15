# Cross-Contract Calls

> Calling functions on other on-chain contracts via `extern onchain mod`
> declarations and the `chain::call` intrinsic. Full semantics in §11.4a.

## Declaring a Callee Interface

A caller declares the foreign contract's public interface at module top level
with `extern onchain mod`:

```sploosh
extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError>;
    pub fn transfer_from(from: Address, to: Address, amount: u256)
        -> Result<(), TokenError>;
}
```

- The block contains only function signatures ending in `;` — no bodies.
- Return types are always `Result<T, E>`.
- The callee's error type (`TokenError` above) must be in scope on the caller
  side via `use` or local definition — caller and callee must agree on error
  enum layout the same way they agree on argument types.
- Declaring an `extern onchain mod` does **not** bind a specific on-chain
  address. The address is passed to `chain::call` at the call site.

## Making a Call

```sploosh
let balance = chain::call(
    token_addr,
    token::balance_of,
    sender
)?;
```

### With Multiple Arguments

Pass a tuple as the arguments value:

```sploosh
chain::call(
    token_addr,
    token::transfer_from,
    (sender, ctx::self_address(), amount)
)?;
```

## Return and Error Semantics

`chain::call` returns `Result<T, ChainError>` — distinct from the callee's
own `Result<T, TokenError>`. `?` on the outer result propagates `ChainError`
to the caller's surrounding function:

```sploosh
@error
pub enum ChainError {
    Reverted { data: Vec<u8> },
    OutOfGas,
    Reentrancy,
    InvalidTarget,
    DecodingError,
}
```

- `Reverted { data }` — the callee reverted. `data` is the callee's revert
  payload, bounded by EVM `RETURNDATACOPY` semantics. Authors who know the
  callee's error enum can decode `data` via the `@error`-generated decoder.
- `OutOfGas` — the callee exhausted its forwarded gas.
- `Reentrancy` — the callee hit its reentrancy guard
  (see [payable-and-reentrancy.md](./payable-and-reentrancy.md)).
- `InvalidTarget` — the address is not a contract, or has no function
  matching the declared selector.
- `DecodingError` — the callee returned bytes that do not decode as the
  declared `T` (callee and caller disagree on the ABI).

## EVM Call Model

On the EVM target, `chain::call` lowers to an EVM `CALL` opcode:

- **Synchronous.** The caller's execution blocks until the callee returns
  or reverts.
- **Gas forwarding.** The EVM default applies — all remaining gas minus the
  1/64 reserve (EIP-150). Explicit per-call `#[gas_limit]` is deferred to
  v0.5.0.
- **ABI encoding.** Arguments are Solidity-ABI-encoded with a 4-byte
  function selector `keccak256(signature_string)[0..4]` derived from the
  Sploosh signature using Solidity type names (`address`, `uint256`, `bool`,
  `bytes`, `string`, ...). This matches Solidity's selector derivation
  exactly, enabling bidirectional Sploosh ↔ Solidity calls.

**No delegatecall in v0.4.x.** Sploosh does not yet expose the EVM
`DELEGATECALL` opcode. `chain::call` always uses `CALL` semantics (callee
executes in its own storage context). A delegate-call intrinsic is deferred
to v0.5.0.

## SVM Call Model

Solana uses **cross-program invocation (CPI)**. On SVM, `chain::call` and
`extern onchain mod` still compile, but the compiler lowers to a CPI
instruction. The user-level surface (synchronous `Result<T, ChainError>`
return, `?` propagation, argument typing) is preserved so that the same
Sploosh source can target either chain. The on-chain ABI, selector
derivation, and account-passing conventions are SVM-specific and deferred to
the Solana-targeting amendment.

## Distinct from `extern "C"`

Both `extern "C" { ... }` (FFI) and `extern onchain mod X { ... }` are
declaration-only blocks nested under the `extern` keyword, but they differ
in every meaningful way:

| Aspect | `extern "C"` | `extern onchain mod` |
|---|---|---|
| Calling convention | C ABI (platform-specific) | Solidity ABI (EVM) / CPI (SVM) |
| Transport | In-process function call | On-chain transaction |
| Safety model | Compiler safe wrappers | Static typing + runtime revert |
| Error surface | `Result<T, FfiError>` | `Result<T, ChainError>` |
| Allowed in `onchain`? | No (compile error) | Yes |
| Allowed in handlers? | Only `async` form (§8.11a) | N/A |

Misuse — e.g., `extern "C"` inside `onchain`, or `extern onchain mod`
declared inside `extern "C"` — is a compile error (E1107). See §11.4a.

## See Also

- §11.4a — Cross-Contract ABI and Call Semantics
- §4.9 — Foreign Function Interface (`extern "C"`)
- [payable-and-reentrancy.md](./payable-and-reentrancy.md) — reentrancy guard interaction
- [evm-vs-svm.md](./evm-vs-svm.md) — target divergence details
