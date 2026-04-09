# Getting Started with Sploosh

> Install Sploosh, create your first project, and run your first program.

## Installation

<!-- TODO: Add installation instructions once the compiler is available -->

## Creating a New Project

```bash
sploosh new my-project
cd my-project
```

This creates the following structure:

```
my-project/
├── sploosh.toml
└── src/
    └── main.sp
```

## Your First Program

Edit `src/main.sp`:

```sploosh
fn main() -> Result<(), AppError> {
    print("Hello, Sploosh!");
    Ok(())
}
```

## Building and Running

```bash
sploosh build                    # Build for native target
sploosh run                      # Build and run
sploosh build --target wasm      # Build for WebAssembly
```

## Project Manifest

`sploosh.toml` defines your project metadata and dependencies:

```toml
[project]
name = "my-project"
version = "0.1.0"
edition = "2026"

[dependencies]

[targets]
default = "native"
```

## Next Steps

- [Basic Types and Variables](basic-types-and-variables.md) — Learn about Sploosh's type system
- [Functions and Control Flow](functions-and-control-flow.md) — Write functions and use control flow expressions
