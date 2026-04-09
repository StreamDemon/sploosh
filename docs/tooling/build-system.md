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
