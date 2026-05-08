# Build System

> Building, testing, and running Sploosh projects.

## Commands

```bash
sploosh new <name>               # Create a new project
sploosh build                    # Build (default target, default profile)
sploosh build --target native    # Build native binary (LLVM)
sploosh build --target wasm      # Build WebAssembly
sploosh build --target evm       # Build EVM bytecode
sploosh build --target svm       # Build Solana SBF
sploosh build --release          # Build with the `release` profile (alias for `--profile release`)
sploosh build --profile <name>   # Build with the named [profile.<name>] from sploosh.toml
sploosh run                      # Build and run
sploosh test                     # Run tests (uses `test` profile)
sploosh check                    # Type-check without building
sploosh update                   # Refresh sploosh.lock (only command that writes the lockfile)
sploosh tree                     # Print the resolved dependency tree
sploosh --explain <code>         # Print long-form explanation for a diagnostic code
```

## Compiler Flags

The flags below select target, profile, and diagnostic rendering for
`sploosh build`, `sploosh check`, and `sploosh test`. Manifest schema
lives in `LANGUAGE_SPEC.md` §14.1; diagnostic schema lives in §18.

| Flag | Values | Default | Purpose |
|------|--------|---------|---------|
| `--target=<t>` | `native`, `wasm`, `evm`, `svm` | `[targets].default` from manifest, else `native` | Selects the build target (§14.1.5). On-chain prohibitions (§11.1, §12.3) are enforced when target is `evm`/`svm`. |
| `--profile=<name>` | any built-in or custom profile name | `dev` | Selects the `[profile.<name>]` block (§14.1.6). `--release` is shorthand for `--profile=release`. |
| `--release` | flag | off | Shorthand for `--profile=release`. |
| `--error-format=<mode>` | `human`, `json`, `short` | `human` | Selects the §18.5 output rendering. `human` is rustc-style for terminals; `json` is newline-delimited JSON (one record per line) for LLM agents and IDEs; `short` is one line per diagnostic, grep-friendly. |

The `--explain <code>` subcommand prints the long-form explanation for a
single diagnostic code, sourced from the local
`docs/reference/compiler-errors.md` registry. It does not make a network
call — the explanation text is bundled with the compiler binary and
versioned to it. The subcommand is deterministic: same code, same
compiler version, same output.

```bash
sploosh build --error-format=json            # NDJSON output for LLM/IDE consumption
sploosh build --error-format=short 2>&1 | grep error   # terse log processing
sploosh --explain E1101                      # long-form explanation for reentrancy revert
```

## Test Runner Flags

`sploosh test` is the canonical runner. Spec contract lives in §13.3.7
of `LANGUAGE_SPEC.md`; the API surface lives in `docs/stdlib/test.md`.

| Flag | Values | Default | Purpose |
|------|--------|---------|---------|
| `--filter <pat>` | substring | none | Only run tests whose fully-qualified path matches `<pat>` |
| `--exact` | flag | off | Treat `--filter` as exact match instead of substring |
| `--test-threads <N>` | `1..` | core count | Run `N` tests concurrently (1 disables parallelism) |
| `--nocapture` | flag | off | Forward test stdout/stderr to the terminal during the run |
| `--seed <hex>` | hex string | random | Fix the property-test RNG seed for reproduction |
| `--cases <N>` | `1..` | 256 | Override the per-property case count |
| `--format <mode>` | `human`, `json` | `human` | Match `--error-format` (§18.5); `json` is one event per line |

```bash
sploosh test                                      # run all tests
sploosh test --filter parser                      # substring match
sploosh test --filter test_parses_addition --exact
sploosh test --test-threads=1 --seed=0xCAFEBABE   # deterministic
sploosh test --format=json | jq '.'               # machine-readable
```

**Determinism contract.** With `--test-threads=1 --seed=<fixed>`, two
runs of the same source against the same compiler version produce
byte-identical output. This is the contract LLM agents and CI snapshot
tests rely on.

**Exit codes.**

| Code | Meaning |
|------|---------|
| `0` | All tests passed |
| `1` | At least one test failed |
| `2` | Runner error (build failure, no tests matched a `--filter`, etc.) |

## Compilation Pipeline

```
Source (.sp)
    |
    +-- Lexer --> Token Stream
    |
    +-- Parser --> AST
    |
    +-- Type Checker --> Typed AST
    |
    +-- Ownership/Borrow Checker
    |
    +-- IR Lowering --> Sploosh IR
    |
    +-> LLVM Backend --> Native Binary / WASM
    |
    +-> EVM Backend --> Solidity Yul --> EVM Bytecode
    |
    +-> SVM Backend --> Solana SBF
```

## Profile Behaviour Notes

- `overflow-checks` is **frozen `true` for `evm` and `svm` targets**
  regardless of the profile's setting. Setting `overflow-checks = false`
  in a profile only affects `native`/`wasm` builds; on-chain builds
  override it and emit a warning. See §14.1.6 and §4.8.
- Per-target profile overrides (e.g., `[profile.release.evm]`) are not
  permitted in v0.5.3. Use `#[cfg(target = "...")]` (§12.3) and feature
  flags for target-specific code paths.

## Lockfile Interaction

`sploosh build`, `sploosh test`, and `sploosh check` *verify* the
lockfile against the manifest and **never rewrite it**. A manifest
change that the existing `sploosh.lock` cannot satisfy fails the build
with a reserved-slot diagnostic (`E14xx`); the user runs `sploosh
update` to refresh the lockfile. See §14.3 for the full lockfile
contract.

<!-- TODO: Expand with incremental compilation, caching, and parallel build details once implemented -->
