# Runbook: Cross-Target Builds

> Building the same codebase for native, WASM, and on-chain targets.

## Conditional Compilation

Use `#[cfg()]` to write target-specific code:

```sploosh
#[cfg(target = "native")]
pub fn save_to_disk(data: &[u8]) -> Result<(), AppError> {
    fs::write("output.bin", data)?;
    Ok(())
}

#[cfg(target = "wasm")]
pub fn save_to_disk(data: &[u8]) -> Result<(), AppError> {
    Err(AppError::Unsupported { feature: "filesystem".into() })
}
```

## Portable Code

Code using only universal stdlib modules works on all targets:

- `std::math` -- all targets
- `std::crypto` -- all targets
- `std::collections` -- all targets
- `std::chain` -- all targets
- Core types (`Vec`, `Map`, `Set`, `Option`, `Result`) -- all targets

## Feature Flags

```toml
# sploosh.toml
[features]
default = ["json"]
json = []
postgres = ["sploosh_db"]
```

```sploosh
#[cfg(feature = "postgres")]
use sploosh_db::Pool;
```

## Building for Multiple Targets

```bash
sploosh build --target native
sploosh build --target wasm
sploosh build --target evm
```

<!-- TODO: Add workspace-level multi-target build configuration once implemented -->
