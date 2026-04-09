# Runbook: Adding an On-Chain Module

> How to add a smart contract module to an existing Sploosh project.

## Steps

1. **Create the contracts directory:**
   ```
   src/contracts/
   ├── mod.sp
   └── token.sp
   ```

2. **Define the on-chain module** in `src/contracts/token.sp`:
   ```sploosh
   onchain mod token {
       storage {
           balances: Map<Address, u256>,
           total_supply: u256,
       }

       pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError> {
           let sender = ctx::caller();
           let bal = storage::get(&self.balances, sender)?;
           if bal < amount { return Err(TokenError::InsufficientBalance); }
           storage::set(&mut self.balances, sender, bal - amount);
           storage::set(&mut self.balances, to, storage::get(&self.balances, to)? + amount);
           emit Transfer { from: sender, to, amount };
           Ok(())
       }
   }
   ```

3. **Add the module** to `src/contracts/mod.sp`:
   ```sploosh
   pub mod token;
   ```

4. **Configure targets** in `sploosh.toml`:
   ```toml
   [targets]
   default = "native"
   contracts = ["evm"]
   ```

5. **Build for on-chain target:**
   ```bash
   sploosh build --target evm
   ```

6. **Create deployment config** in `deploy/evm.toml`.

<!-- TODO: Expand with deployment steps and verification once implemented -->
