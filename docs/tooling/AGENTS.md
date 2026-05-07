# AGENTS.md — `docs/tooling/`

The (eventual) `sploosh` CLI, the manifest, and editor integration.

## Identity

This folder describes the *toolchain* — separate from the language itself. The compiler doesn't exist yet, but the contract for what it should accept lives here.

## Files

| File | Purpose |
|---|---|
| `build-system.md` | `sploosh build`, target selection, output layout |
| `package-management.md` | Dependencies, registries, version resolution |
| `sploosh-toml.md` | Manifest schema (`[package]`, `[dependencies]`, `[targets]`, etc.) |
| `editor-setup.md` | LSP / editor integration |

## Patterns & Conventions

- **Forward-looking.** These docs describe the intended toolchain. Mark unimplemented sections clearly (`> TODO: not implemented in v0.5.x`) so readers don't think the tool already exists.
- **`sploosh.toml` is the manifest** — schema lives in `sploosh-toml.md`. Examples elsewhere (e.g., `docs/runbooks/new-project-setup.md`, `docs/examples/`) must use the same schema.
- **CLI flags get a table** — flag, type, default, description.
- **Cross-target builds** are first-class: `--target native|wasm|evm|svm`. Each target has invariants (e.g., on-chain prohibitions); link to `docs/web3/` instead of restating.

## Touch Points

- `build-system.md` — `sploosh build` invocation, output paths, target validation.
- `sploosh-toml.md` — the source of truth for manifest fields; all examples must match.
- `package-management.md` — pre-1.0; expect heavy churn.
- `editor-setup.md` — LSP integration is aspirational; keep it short until a real LSP exists.

## JIT Index Hints

```pwsh
# Find every `sploosh.toml` example
rg -n "\[package\]|\[dependencies\]" docs

# Find every CLI invocation
rg -n "sploosh build|sploosh new|sploosh test" docs
```

## Common Gotchas

- **CLI flags don't exist yet.** Don't promise behaviour the spec doesn't endorse. When in doubt, mark `TODO`.
- **`package-management.md` is the most aspirational page.** Don't write code-as-if-it-existed; describe the model.
- **Editor setup** depends on an LSP that doesn't exist. Keep `editor-setup.md` aligned with reality.

## Pre-PR Checks

```pwsh
# Tooling and runbooks must agree on commands
git diff --name-only main...HEAD | findstr -r "tooling runbooks examples"

# Confirm sploosh.toml examples are valid against the schema
rg -n "\[package\]" docs/examples docs/runbooks
```
