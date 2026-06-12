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

- **One example = one focused walkthrough.** Each page shows the core source file(s) (`main.sp`, or a module like `contracts/token.sp`) plus a "Key Patterns" section. Examples are deliberately minimal and single-file — full project scaffolding (`sploosh.toml`, build/run commands) appears only where it earns its space (`hello-world.md`); link to `docs/tooling/` and `docs/guide/` for the rest.
- **Idiomatic Sploosh throughout.** These show how Sploosh *should* be written — error handling via `Result<T, E>`, `?` propagation, attributes where appropriate.
- **Make the target clear where it matters.** `token-contract.md` is the on-chain example; the rest are native. A formal targets-up-front declaration is aspirational for new examples, not yet present in existing ones.
- **No half-implementations.** If something is omitted for brevity, say so explicitly with `// ... (omitted)`.

## Touch Points

- `hello-world.md` — the "first 30 seconds" of Sploosh. Keep it short, complete, and copy-pasteable.
- `actor-chat-server.md` — canonical actor example. Cross-references spec §8 and `docs/guide/actors-and-concurrency.md`.
- `token-contract.md` — canonical on-chain example. Cross-references §11 and `docs/web3/`. Must respect on-chain prohibitions (no floats, no actors, etc.).
- `rest-api.md` — uses `std::web` + `std::db` + `std::json` (native only).

## JIT Index Hints

```pwsh
# Find which examples cover a feature
rg -l "spawn|actor|onchain|@payable" docs/examples

# Find the walkthrough sections
rg -n "## Key Patterns" docs/examples

# Sweep examples for spec violations
rg -n "f64\.sqrt|@fast_math" docs/examples/token-contract.md   # should match nothing
```

## Common Gotchas

- **`token-contract.md` cannot use floats, actors, FFI, or fs/net.** All on-chain prohibitions apply.
- **Actors spawn via `spawn Actor::init(...)`** — no closure involved. `move` closures (§4.6) are required only when capturing into a `spawn async { }` task.
- **`?` propagation requires the surrounding fn to return `Result<_, _>`.** Don't accidentally show `?` in a `fn main() -> ()`.
- **`sploosh.toml` is the manifest** — see `docs/tooling/sploosh-toml.md` for the schema.

## Pre-PR Checks

```pwsh
# Examples must reflect current spec — verify both moved together
git diff --name-only main...HEAD | findstr -r "examples spec-plans"

# Verify on-chain examples don't reference forbidden APIs
rg -n "(fs|net|io|db|web|env|log|time|test|spawn|actor)::" docs/examples/token-contract.md
```
