# Storage and State

> Declaring persistent on-chain state with `storage` blocks.

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

<!-- TODO: Expand with storage layout details, gas costs, and optimization patterns once implemented -->
