# Runbook: Deploying to EVM

> End-to-end deployment walkthrough for Ethereum Virtual Machine targets.

## Prerequisites

- Sploosh compiler installed
- Target chain RPC endpoint
- Funded deployer account

## Steps

1. **Build for EVM:**
   ```bash
   sploosh build --target evm --release
   ```

2. **Configure deployment** in `deploy/evm.toml`:
   <!-- TODO: Document deployment config format -->

3. **Deploy:**
   <!-- TODO: Document deployment command and verification -->

4. **Verify contract:**
   <!-- TODO: Document verification process -->

## Off-Chain Integration

```sploosh
offchain fn interact() -> Result<(), AppError> {
    let contract = Contract::connect("0xDEPLOYED_ADDRESS")?;
    let result = contract.call(token::balance_of, address).await?;
    Ok(())
}
```

<!-- TODO: Full walkthrough once deployment tooling is implemented -->
