# AGENTS.md — `docs/runbooks/`

Operational how-tos: scaffolding a project, debugging an actor, deploying a contract, upgrading the spec.

## Identity

Step-by-step procedures. Each runbook answers "I want to do X — give me the commands."

## Files

```
new-project-setup.md           adding-onchain-module.md
cross-target-builds.md         actor-debugging.md
debugging-ownership-errors.md  testing-strategies.md
deploying-to-evm.md            deploying-to-solana.md
upgrading-sploosh-version.md
```

## Patterns & Conventions

- **Numbered steps, copy-pasteable commands.** Prose between steps stays minimal.
- **Pre-conditions stated up front.** "You have a working `sploosh` toolchain installed and `sploosh.toml` configured."
- **Failure modes & recovery.** Each runbook has a "If something goes wrong" section that maps observed errors → fixes.
- **Tool-version specific.** When a step depends on a specific tool version (e.g., `forge`, `solana-cli`), pin it.

## Touch Points

- `new-project-setup.md` is the entry point — keep it aligned with `docs/tooling/sploosh-toml.md` and `docs/examples/hello-world.md`.
- `actor-debugging.md` references supervision strategies (§8) — sync when supervisor semantics change.
- `debugging-ownership-errors.md` mirrors `docs/reference/compiler-errors.md` for ownership-related diagnostics.
- `cross-target-builds.md` covers `sploosh build --target {native,wasm,evm,svm}` — sync with `docs/tooling/build-system.md`.

## JIT Index Hints

```pwsh
# Find every runbook that touches a target
rg -l "evm|svm|wasm|native" docs/runbooks

# Find diagnostic codes called out in runbooks
rg -n "E\d{4}" docs/runbooks
```

## Common Gotchas

- **Runbooks lag the spec.** When a behaviour changes (e.g., new error code, new attribute), runbooks often miss the update. When changing the spec, sweep `docs/runbooks/`.
- **Solana deploy runbook is short on purpose** — SVM specifics are still being amended in the spec. Don't add detail the spec doesn't yet endorse.
- **Don't duplicate the build-system docs.** Runbooks reference `docs/tooling/build-system.md`; they don't restate it.

## Pre-PR Checks

```pwsh
# Runbook commands should not have stale flags
rg -n "sploosh build" docs/runbooks

# Verify runbooks didn't drift from tooling docs
git diff --name-only main...HEAD | findstr -r "runbooks tooling"
```
