# AGENTS.md — `docs/examples/`

End-to-end example projects, presented as walkthroughs. Each example is a complete, self-contained program.

## Identity

Showpiece projects. They demonstrate idiomatic Sploosh across the four targets: native CLI, REST API (native), actor system (native), token contract (EVM/SVM).

## Files

```
hello-world.md            cli-tool.md
rest-api.md               actor-chat-server.md
token-contract.md
```

## Patterns & Conventions

- **One example = one runnable program.** Show the full file structure (`src/main.sp`, `sploosh.toml`) and the build/run commands.
- **Idiomatic, not minimal.** These show how Sploosh *should* be written — error handling via `Result<T, E>`, `?` propagation, attributes where appropriate.
- **Targets declared up-front.** Each example states its target(s) in the first paragraph (`native`, `evm`, etc.).
- **No half-implementations.** If something is omitted for brevity, say so explicitly with `// ... (omitted)`.

## Touch Points

- `hello-world.md` — the "first 30 seconds" of Sploosh. Keep it short, complete, and copy-pasteable.
- `actor-chat-server.md` — canonical actor + channel example. Cross-references spec §8 and `docs/guide/actors-and-concurrency.md`.
- `token-contract.md` — canonical on-chain example. Cross-references §11 and `docs/web3/`. Must respect on-chain prohibitions (no floats, no actors, etc.).
- `rest-api.md` — uses `std::net` + `std::web` (native only).

## JIT Index Hints

```pwsh
# Find which examples cover a feature
rg -l "spawn|actor|onchain|@payable" docs/examples

# Confirm targets are declared
rg -n "Target:|Targets:" docs/examples

# Sweep examples for spec violations
rg -n "f64\.sqrt|@fast_math" docs/examples/token-contract.md   # should match nothing
```

## Common Gotchas

- **`token-contract.md` cannot use floats, actors, FFI, or fs/net.** All on-chain prohibitions apply.
- **Actor examples must use `move` closures** when spawning, per §6.
- **`?` propagation requires the surrounding fn to return `Result<_, _>`.** Don't accidentally show `?` in a `fn main() -> ()`.
- **`sploosh.toml` is the manifest** — see `docs/tooling/sploosh-toml.md` for the schema.

## Pre-PR Checks

```pwsh
# Examples must reflect current spec — verify both moved together
git diff --name-only main...HEAD | findstr -r "examples spec-plans"

# Verify on-chain examples don't reference forbidden APIs
rg -n "(fs|net|io|db|web|env|spawn|actor)::" docs/examples/token-contract.md
```
