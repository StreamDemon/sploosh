/// FFI and on-chain surfaces: extern "C" blocks (§4.9), string-target async
/// extern blocks, extern onchain mod declarations (§11.4a), a top-level
/// onchain enum, and an onchain mod with a storage block (§11.1a).
extern "C" {
    pub fn puts(s: &str) -> i32;
    fn c_open(path: &str, flags: i32) -> Result<i32, FfiError>;
}

extern "C" async {
    fn poll_events(mask: u32) -> u64;
}

extern onchain mod token {
    pub fn balance_of(account: Address) -> Result<u256, TokenError>;
}

onchain enum TokenError {
    InsufficientBalance,
    Unauthorized,
}

onchain mod vault {
    storage {
        balances: Map<Address, u256>,
        supply: u256,
    }

    pub fn deposit(amount: u256) -> Result<u256, TokenError> {
        let caller = ctx::caller();
        let current = storage::get(&self.balances, caller)?;
        storage::set(&mut self.balances, caller, current + amount);
        Ok(current + amount)
    }
}
