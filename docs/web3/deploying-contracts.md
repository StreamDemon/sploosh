# Deploying Contracts

> Building and deploying on-chain modules to EVM and SVM targets.

## Build for Target

```bash
sploosh build --target evm    # EVM bytecode
sploosh build --target svm    # Solana SBF
```

## Deployment Configuration

Deployment config lives in `deploy/`:

```
deploy/
└── evm.toml        # Chain deployment config
```

## Off-Chain Interaction

```sploosh
offchain fn check_balance(user: Address) -> Result<u256, AppError> {
    let contract = Contract::connect("0x1234...")?;
    let balance = contract.call(token::balance_of, user).await?;
    Ok(balance)
}
```

<!-- TODO: Expand with full deployment walkthrough, network configuration, and verification once tooling is built -->
