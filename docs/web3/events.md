# Events

> Emitting on-chain events for off-chain indexing and logging.

## Defining Events

```sploosh
onchain enum Event {
    Transfer { from: Address, to: Address, amount: u256 },
    Approval { owner: Address, spender: Address, amount: u256 },
    Deposit { sender: Address, amount: u256 },
}
```

## Emitting Events

```sploosh
emit Transfer { from: sender, to, amount };
```

Events are emitted during transaction execution and stored in transaction logs for off-chain consumers to index.

<!-- TODO: Add indexing patterns and off-chain event listening examples -->
