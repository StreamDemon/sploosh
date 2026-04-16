# Build System

> Building, testing, and running Sploosh projects.

## Commands

```bash
sploosh new <name>               # Create a new project
sploosh build                    # Build (default target)
sploosh build --target native    # Build native binary (LLVM)
sploosh build --target wasm      # Build WebAssembly
sploosh build --target evm       # Build EVM bytecode
sploosh build --target svm       # Build Solana SBF
sploosh build --release          # Release build (optimized)
sploosh run                      # Build and run
sploosh test                     # Run tests
sploosh check                    # Type-check without building
sploosh --explain <code>         # Print long-form explanation for a diagnostic code
```

## Compiler Flags

The diagnostic rendering flags select how `sploosh build`, `sploosh check`,
and `sploosh test` present compiler output. The *schema* of a diagnostic
is specified in `LANGUAGE_SPEC.md` §18; this page owns only the flag
surface.

| Flag | Modes | Default | Purpose |
|------|-------|---------|---------|
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

<!-- TODO: Expand with incremental compilation, caching, and parallel build details once implemented -->
