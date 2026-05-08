# AGENTS.md — `docs/migration/`

"Coming from X" guides. Audience: experienced developers in a neighbouring language deciding whether Sploosh is worth their time.

## Identity

Translation guides for users with prior fluency. Each file maps idioms from a source language to Sploosh equivalents and calls out the gaps.

## Files

```
from-rust.md         from-elixir.md
from-solidity.md     from-typescript.md
```

## Patterns & Conventions

- **Side-by-side examples.** Two code blocks: source language on the left/top, Sploosh on the right/bottom.
- **Honest about differences.** Don't oversell. If Sploosh is missing a feature (e.g., no `Rc`/`Arc`, no delegatecall), say so.
- **Map the surprises.** Each guide should have a "Things that will surprise you" section.
- **Concise.** These are reference cards, not tutorials. Send readers to `docs/guide/` for depth.

## Touch Points

- `from-rust.md` — closest neighbour. Highlight: no `unsafe`, no `Rc`/`Arc`, default-checked arithmetic, single ownership only, actors instead of `Send + Sync` shared state, `Shared<T>` (§4.4a) as the only opt-in for shared immutable data.
- `from-elixir.md` — actor model is similar, but Sploosh is statically typed and uses Rust-style ownership inside actors.
- `from-solidity.md` — `onchain mod` ≈ contract; storage layout is Solidity-compatible by design (§11.1a). Reentrancy guard is built-in; no manual `nonReentrant` modifier needed.
- `from-typescript.md` — short by design. TypeScript users should mostly be sent to `docs/guide/getting-started.md`.

## JIT Index Hints

```pwsh
# Confirm the migration mentions a recently changed feature
rg -n "Shared<T>|Reentrancy|@reentrant" docs/migration

# Find code-block language tags (should be `rust`, `elixir`, `solidity`, `typescript`, or `sploosh`)
rg -n '```(rust|elixir|solidity|typescript|sploosh)' docs/migration
```

## Common Gotchas

- **Don't import language anti-patterns.** A migration guide should not justify a Sploosh idiom by appealing to the source language's quirks.
- **No `unsafe` analogue.** Rust users will look for it; the answer is FFI via `extern "C"` with safe wrappers (§4.9).
- **No `panic!` in safe code** — Sploosh aborts on integer overflow and supervisor-restartable actor failures, but there's no user-facing `panic!` keyword.

## Pre-PR Checks

```pwsh
# Migration changes should reference current spec semantics
git diff --name-only main...HEAD | findstr "migration"

# Verify the Sploosh code blocks use the right tag
rg -n '```sploosh' docs/migration
```
