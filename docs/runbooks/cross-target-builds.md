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

- `std::math` (integer methods) -- all targets
- `std::math` (float methods, `@fast_math`) -- native and wasm only (§13.2)
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
postgres = ["dep:sploosh_db"]
```

```sploosh
#[cfg(feature = "postgres")]
use sploosh_db::Pool;
```

## Target-Specific Dependencies

`[target.<target>.dependencies]` declares deps that apply only when
building for the named target. Sections are merged additively with the
base `[dependencies]` table at resolution time:

```toml
[dependencies]
sploosh_web = "0.3"

[target.wasm.dependencies]
# Wasm builds use the lighter client-only feature set
sploosh_web = { version = "0.3", default-features = false, features = ["client"] }

[target.evm.dependencies]
sploosh_chain = { version = "0.2", features = ["evm"] }

[target.svm.dependencies]
sploosh_chain = { version = "0.2", features = ["svm"] }
```

Conflicting `version`/`source` between base and target sections is a
manifest error. Only `features` and `default-features` may differ. The
on-chain stdlib prohibitions in §11.1 / §12.3 still apply — declaring a
dep under `[target.evm.dependencies]` does not bypass them.

## Building for Multiple Targets

```bash
sploosh build --target native
sploosh build --target wasm
sploosh build --target evm
sploosh build --target svm
```

`overflow-checks` is **frozen `true` for `evm` and `svm` targets**
regardless of profile setting; on-chain builds emit a warning if a
profile attempted to disable it. See §14.1.6.

<!-- TODO: Add workspace-level multi-target build configuration once implemented -->
