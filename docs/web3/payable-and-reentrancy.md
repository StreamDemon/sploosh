# Payable Functions and Reentrancy

> Receiving native tokens and the on-chain reentrancy guard. Mechanism is
> specified in §11.3a of the language specification.

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

On-chain functions are **non-reentrant by default**. A function cannot be
called again while it is already executing. This prevents the most common
class of smart contract vulnerabilities — callback-based recursion through a
malicious or buggy external contract.

To opt in to reentrancy on a specific function (discouraged):

```sploosh
@reentrant
pub fn peek_balance(who: Address) -> u256 {
    storage::get(&self.balances, who).unwrap_or(0)
}
```

## Guard Mechanism

Every `onchain mod` maintains a single **per-contract reentrancy flag** in
transient runtime state (not persisted across transactions).

- On entry to any `pub` on-chain function not marked `@reentrant`, the
  runtime checks the flag. If already set, the call reverts with
  `ChainError::Reentrancy` and all state changes in the current call frame
  are unwound.
- If the flag is clear, the runtime sets it, executes the function body, and
  clears it on return — whether the function returns `Ok`, returns `Err`, or
  reverts. The clear-on-revert rule means a revert does not leave the guard
  stuck set.
- A function marked `@reentrant` **skips both the check and the set**. The
  flag is neither consulted nor modified on entry or exit.

**`@reentrant` scope.** The attribute disables the guard for the marked
function only. Guarded sibling functions in the same contract continue to
observe and set the flag.

## Cross-Contract Interaction

The guard is what makes cross-contract callbacks safe by default:

```sploosh
onchain mod vault {
    storage {
        balances: Map<Address, u256>,
    }

    pub fn withdraw(amount: u256) -> Result<(), VaultError> {
        // Guarded by default. If the callee (a wallet contract below) calls
        // back into withdraw() or any other guarded function in this vault,
        // the call reverts with ChainError::Reentrancy.
        let sender = ctx::caller();
        let bal = storage::get(&self.balances, sender).unwrap_or(0);
        if bal < amount { return Err(VaultError::Insufficient); }
        storage::set(&mut self.balances, sender, bal - amount);
        chain::call(sender, wallet::on_receive, amount)?;
        Ok(())
    }
}
```

If contract A's function `foo` (guarded) calls contract B, and B tries to
call back into A's function `bar`:

- If `bar` is **not** `@reentrant`, the call reverts with
  `ChainError::Reentrancy`.
- If `bar` **is** `@reentrant`, the call proceeds. Authors who mark `bar`
  `@reentrant` are responsible for its safety under concurrent invocation of
  the same contract.

## Gas Cost

The guard is backed by transient execution state (see the "transient runtime
state" note above), not persistent storage. On EVM, it lowers to **transient
storage** — `TLOAD` / `TSTORE` (EIP-1153) on Cancun and later hard forks —
adding one `TLOAD` on entry plus one `TSTORE` on entry and exit of every
non-`@reentrant` `pub` function, priced per the active hard fork's gas
schedule. Pre-1153 forks are not supported target environments: §11.3a
requires the mechanism to be unwound automatically on transaction revert,
which transient storage provides natively. Exact costs vary by hard fork;
the overhead is small compared with most real-world contract logic but not
free.

## Distinct from Actor SelfCall

The on-chain reentrancy guard and the §8.10.1 actor `SelfCall` detection
share the word "reentrancy" but are **different concepts at different
layers**:

- **Actor `SelfCall`** is a runtime check in the scheduler that catches an
  actor handler synchronously requesting a reply from its own mailbox — a
  deadlock condition specific to the actor scheduler.
- **On-chain guard** is a per-contract flag that catches a cross-contract
  callback re-entering a guarded function — a vulnerability class specific
  to the EVM call model.

On-chain execution has no actor scheduler, and actor execution has no
storage slots, so the two mechanisms never overlap.

## See Also

- §11.3a — Reentrancy Guard Mechanism
- §8.10.1 — Actor re-entrant calls and deadlock
- [cross-contract-calls.md](./cross-contract-calls.md) — how `chain::call` interacts with the guard
- §12.1 — `@payable` and `@reentrant` attribute reference
