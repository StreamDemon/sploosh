# AGENTS.md — `docs/reference/`

Mirror of authoritative spec sections. **These files restate the spec; they don't decide.** When in doubt, defer to `docs/spec-plans/LANGUAGE_SPEC.md`.

## Identity

Per-topic quick-references derived from the spec. Used by IDE-style lookups and by anyone who doesn't want to read a 180KB document.

## Files & Spec Mapping

| File | Mirrors |
|---|---|
| `grammar.md` | `LANGUAGE_SPEC.md` §16 (EBNF) |
| `keywords.md` | §2.3 (39 keywords) |
| `attributes.md` | §12 (`@test`, `@derive`, `@inline`, `@error`, `@payable`, `@reentrant`, `@supervisor`, `@mailbox`, `@overflow`, `@fast_math`) |
| `operator-precedence.md` | §2.4 |
| `type-conversion-rules.md` | §3 / §4 (`as` rules) |
| `compiler-errors.md` | §18 |

## Patterns & Conventions

- **Restate, don't redesign.** If the reference and the spec disagree, the spec wins and the reference is wrong.
- **Examples should be minimal** — one-liners that show the rule, not full programs. Save prose for `docs/guide/`.
- **Tables over prose** where the data is tabular (precedence, keyword categories, error codes).

## Touch Points

- `compiler-errors.md` is the most likely to drift — diagnostic codes get added during spec work and the catalog must keep up.
- `attributes.md` has subtle on-chain prohibitions (`@fast_math` is a compile error inside `onchain`); cross-check with `docs/web3/`.
- `grammar.md` is generated mentally from §16 — if you tweak grammar, prefer copy-paste from §16.

## JIT Index Hints

```pwsh
# Find usage of a keyword across docs
rg -n "\b(actor|spawn|onchain|emit)\b" docs

# Find diagnostic codes
rg -n "E\d{4}" docs/reference/compiler-errors.md

# Spot drift between reference and spec for a single keyword
rg -n -A 3 "send" docs/reference/keywords.md docs/spec-plans/LANGUAGE_SPEC.md
```

## Common Gotchas

- `as` is **numeric-only** (§3). Reference must not imply otherwise.
- Lifetimes use single-source elision; multi-source ones must be explicit. Reference must mirror §4 wording.
- The keyword list is **exactly 39** (§2.3). Adding a keyword is a spec-level decision and ripples into `keywords.md`, the EBNF, and the lexer chapter.

## Pre-PR Checks

```pwsh
# Confirm spec mirror was also touched
git diff --name-only main...HEAD | findstr "spec-plans"

# Spot-check the count of keywords (must equal 39 — §2.3)
rg -c '^\| `' docs/reference/keywords.md
```
