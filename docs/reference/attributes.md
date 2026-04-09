# Attributes and Compiler Directives

> `@` attributes and `#[]` compiler directives.

## Standard Attributes

| Attribute | Purpose |
|-----------|---------|
| `@test` | Mark a function as a test case |
| `@derive(...)` | Auto-generate trait impls |
| `@error` | Auto-generate `From`, `Display`, `Error` for error enums |
| `@inline` | Hint to inline a function |
| `@payable` | On-chain: function accepts native tokens |
| `@reentrant` | On-chain: opt-in to reentrancy (discouraged) |
| `@supervisor(...)` | Mark an actor as a supervisor |

## Derive Macros

| Derive | Generates |
|--------|-----------|
| `Debug` | `Debug` trait (for `{:?}`) |
| `Clone` | `Clone` trait (deep copy) |
| `Copy` | `Copy` trait (bitwise copy, requires `Clone`) |
| `Eq` | `Eq` trait (structural equality) |
| `Hash` | `Hash` trait |
| `Serialize` | `Serialize` trait |
| `Deserialize` | `Deserialize` trait |
| `Ord` | `Ord` trait (total ordering) |

## Compiler Directives

```sploosh
#[target(evm)]         // Compile only for EVM target
#[target(native)]      // Compile only for native target
#[gas_limit(50000)]    // On-chain gas budget
#[cfg(test)]           // Include only in test builds
#[cfg(debug)]          // Include only in debug builds
```

## cfg Flags

| Flag | True when |
|------|-----------|
| `#[cfg(test)]` | Running `sploosh test` |
| `#[cfg(debug)]` | Building without `--release` |
| `#[cfg(release)]` | Building with `--release` |
| `#[cfg(target = "native")]` | `--target native` |
| `#[cfg(target = "wasm")]` | `--target wasm` |
| `#[cfg(target = "evm")]` | `--target evm` |
| `#[cfg(target = "svm")]` | `--target svm` |
| `#[cfg(feature = "name")]` | Feature enabled in `sploosh.toml` |
