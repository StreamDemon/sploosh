# Events

> Emitting on-chain events for off-chain indexing and logging.

## Defining Events

```sploosh
onchain enum Event {
    Transfer {
        #[indexed] from: Address,
        #[indexed] to: Address,
        amount: u256,
    },
    Approval {
        #[indexed] owner: Address,
        #[indexed] spender: Address,
        amount: u256,
    },
    Deposit { sender: Address, amount: u256 },
}
```

## Emitting Events

```sploosh
emit Transfer { from: sender, to, amount };
```

Events are emitted during transaction execution and stored in transaction
logs for off-chain consumers to index. Emits made during a transaction that
subsequently reverts are discarded along with all other state changes
(§11.7a).

## Indexed Fields (`#[indexed]`)

On EVM, an event field marked `#[indexed]` becomes an indexed topic in the
emitted log, letting off-chain indexers filter by that field cheaply. The
EVM reserves four topic slots per log entry: topic 0 is always the event
signature hash, leaving topics 1–3 available for indexed fields. Up to
**three** `#[indexed]` fields per event variant are allowed on EVM; a
variant with more than three is a compile error (E1105).

Unmarked fields are packed into the event's data region (unindexed).

On SVM, `#[indexed]` is accepted for source-compatibility but is a no-op at
the Solana log record level — Solana programs emit a single data buffer per
log entry. Off-chain indexers for SVM contracts must parse the full event
payload from the data region.

## Off-Chain Indexing

An off-chain indexer (e.g., a Sploosh `offchain` service, The Graph
subgraph, or a Solidity-ABI-compatible indexer) subscribes to a contract's
event topics and decodes each matching log. For EVM contracts, the Sploosh
compiler emits an event ABI alongside the deployed bytecode so indexers can
decode the topic/data split for each variant.

Concrete indexer integration patterns (subgraph manifests, block-by-block
replay, reorg handling) are deployment-tooling concerns and will be covered
in a future revision of this page alongside the deployment runbooks.

## See Also

- §11.5 — Events and `#[indexed]` in the language specification
- §12.3 — `#[indexed]` directive reference
- [docs/web3/onchain-overview.md](./onchain-overview.md) — `emit` as a compiler intrinsic
