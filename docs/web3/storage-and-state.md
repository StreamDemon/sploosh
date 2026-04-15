# Storage and State

> Declaring persistent on-chain state with `storage` blocks. Full layout
> semantics are specified in §11.1a of the language specification.

## Storage Declaration

```sploosh
onchain mod token {
    storage {
        balances: Map<Address, u256>,
        total_supply: u256,
        owner: Address,
    }
}
```

## Reading and Writing Storage

```sploosh
let balance = storage::get(&self.balances, sender)?;
storage::set(&mut self.balances, sender, new_balance);
```

`storage::get` and `storage::set` are compiler intrinsics (§13.0). They are
only usable inside `onchain` modules, and the target backend lowers each call
to the target's native persistence primitive (EVM `SLOAD` / `SSTORE`, SVM
account reads and writes).

## Layout Model

Sploosh specifies storage as a **target-pluggable abstraction**. At the
Sploosh source level:

- `storage { ... }` fields are opaque **persistent locations**.
- `Map<K, V>` keys resolve to **derived** persistent locations.
- The same source compiles unchanged to every on-chain target.

The concrete bytes on chain differ per target. The reference realization is
the EVM layout, which Sploosh adopts verbatim from Solidity.

## EVM Layout (Reference Realization)

Sploosh adopts the Solidity storage layout verbatim on EVM so that Sploosh
contracts interoperate with existing Solidity contracts (ABIs, upgrade
proxies, indexers) without a translation layer.

### Struct fields

Fields in a `storage { ... }` block occupy **sequential 32-byte slots** in
declaration order, starting at slot `0`. Within a slot, primitives that fit
are right-aligned and packed in declaration order — identical to Solidity's
packing rules. A field that would overflow the remaining same-slot space is
promoted to a fresh slot.

```sploosh
storage {
    admin: Address,         // slot 0, bytes 0..20 (high 12 bytes zero)
    paused: bool,           // slot 0, byte 20
    fee_bps: u16,           // slot 0, bytes 21..23
    total_supply: u256,     // slot 1 (full slot)
    balances: Map<Address, u256>,  // slot 2 (header; entries are derived)
}
```

### `Map<K, V>`

A `Map<K, V>` field occupies one slot `s` as the map's **header slot** — its
contents are computed, not stored there. For a given key `k`, the value lives
at:

```
slot = keccak256(abi.encode(k, s))
```

where `abi.encode` is Solidity's ABI encoding of the key type, padded to 32
bytes. `Address` keys pad to 32 bytes with 12 leading zero bytes; integer
keys are zero-extended big-endian.

### Nested maps

`Map<K1, Map<K2, V>>` recurses. With outer slot `s`, the inner map's slot is
`keccak256(abi.encode(k1, s))`, and the final value lives at
`keccak256(abi.encode(k2, keccak256(abi.encode(k1, s))))`. Deeper nestings
apply the same rule.

### Dynamic types

`Vec<T>` and `String` store their **length** in their declared slot `s`; the
element region begins at `keccak256(s)` and grows contiguously. This matches
Solidity's `T[]` and `string` encoding.

### Fixed arrays

`[T; N]` occupies `ceil(N * size_of::<T>() / 32)` slots inline. Fixed arrays
do **not** use `keccak256` derivation.

### Per-contract isolation

Distinct `onchain mod` declarations compile to distinct contracts with
independent storage roots. A contract cannot read another contract's raw
storage; inter-contract state is accessed only through cross-contract calls
(see [cross-contract-calls.md](./cross-contract-calls.md)).

## SVM Layout

Solana uses an **account-based** persistence model: program state lives in
one or more on-chain accounts whose layouts the program chooses. SVM does
not derive slots via `keccak256`. The Sploosh surface (`storage { ... }`
declarations, `storage::get` / `storage::set` intrinsics, `Map<K, V>` field
type) is identical to EVM, but the concrete SVM layout — account schema,
borsh / Anchor-compatible serialization, rent accounting — is deferred to a
future amendment targeting Solana deployment. Contracts authored against
`storage` today should expect the final SVM layout to be a Sploosh-defined
schema over one or more program accounts, not a direct port of the EVM slot
derivation.

## Determinism and Ordering

All storage operations are deterministic: the same sequence of
`storage::get` / `storage::set` calls on the same input produces the same
bytes on chain. Sploosh does not introduce a hidden cache or reorder writes;
the order of storage effects matches source order within each transaction.
Costs for `SLOAD` / `SSTORE` on EVM are priced per the active hard fork's
gas schedule (see [evm-vs-svm.md](./evm-vs-svm.md) and §11.7a).

## See Also

- §11.1a — Storage Layout specification
- §13.0 — `storage::get` / `storage::set` intrinsic signatures
- §11.7a — Gas model pricing storage operations
