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
| `@reentrant` | On-chain: opt-in to reentrancy. Disables the §11.3a per-contract guard for this function only — the flag is neither consulted nor modified on entry or exit. Does not alter guard state observed by other non-`@reentrant` functions in the same contract. |
| `@supervisor(...)` | Mark an actor as a supervisor. Accepts `strategy`, `max_restarts`, `window_secs` (sliding window). Restart runs a fresh `init` with stored arguments; old handles become permanently dead — see §8.7a of the spec. Compile error inside `onchain`. |
| `@mailbox(capacity: N)` | Set actor mailbox capacity (default 1024). Mailbox stays locked across `.await` points in handlers (§8.10); full mailbox blocks senders, and blocked senders wake with `Err(SendError::Dead)` if the destination dies (§8.11). Compile error inside `onchain`. |
| `@overflow(wrapping)` | Opt function into wrapping arithmetic (compile error on-chain) |
| `@fast_math(flags)` | Enable LLVM fast-math flags for floating-point operations in the function body. Flags: `contract`, `afn`, `reassoc`, `arcp`, `nnan`, `ninf`, `nsz`. Bare `@fast_math` = `@fast_math(contract, afn)`. Per-function scope, not inherited. Compile error inside `onchain`. |

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
#[gas_limit(50000)]    // On-chain gas budget (EVM only; advisory)
#[indexed]             // Event variant field marker (EVM topic slot)
#[cfg(test)]           // Include only in test builds
#[cfg(debug)]          // Include only in debug builds
```

**`#[gas_limit(N)]`** — EVM-only; advisory annotation surfaced in the
deployed contract's ABI metadata. Does **not** cap runtime execution (runtime
OOG is produced by the EVM). Compile error on SVM, native, and wasm. See
§11.7a.

**`#[indexed]`** — marks an event variant field as an indexed topic on EVM
(up to three per variant; topic 0 is the event signature hash). Applying
`#[indexed]` outside an event variant field is a compile error. On SVM,
accepted for source-compatibility but a no-op at the Solana log record
level. See §11.5.

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
