# AGENTS.md — `docs/spec-plans/`

The authoritative root of the Sploosh language. Edit with extreme care.

## Identity

This folder holds the language specification, its prompt-sized mirror, and the latest review notes. Everything else in `docs/` derives from these files.

## Files

| File | Purpose |
|---|---|
| `LANGUAGE_SPEC.md` | **Authoritative spec.** Sole source of truth. |
| `LANGUAGE_SPEC_PROMPT_CORE.md` | Condensed LLM-facing reference for the language core (excluding §11). Soft target ~4,000 cl100k_base tokens. |
| `LANGUAGE_SPEC_PROMPT_WEB3.md` | Condensed LLM-facing reference for §11 on-chain surface. Soft target ~1,500 cl100k_base tokens. |
| `LANGUAGE_SPEC_PROMPT.md` | Retired combined prompt-mirror; redirects to the `_CORE` + `_WEB3` files. |
| `LANGUAGE_SPEC_REVIEW.md` | Review notes / open issues / proposed amendments. |

## Patterns & Conventions

- **Section numbering matters.** Sections are referenced from many other docs as `§3`, `§4.4a`, `§11.4a`, `§16`, etc. Renumbering is a cross-tree change — search before you renumber.
- **Appendix D is the changelog.** Every material spec change adds an Appendix D entry with the version bump (e.g., `v0.5.2`).
- **Design Decisions Log (§17)** captures rationales for non-obvious choices. Add an entry when you make a controversial call.
- **The prompt-mirror is split** as of v0.5.8 — `LANGUAGE_SPEC_PROMPT_CORE.md` (~4K `cl100k_base` tokens soft target) and `LANGUAGE_SPEC_PROMPT_WEB3.md` (~1.5K soft target), per §1 principle 7. Adding to one section means pruning another. The combined `LANGUAGE_SPEC_PROMPT.md` is a redirect-only stub.
- **Code fences in spec use `sploosh`** as the language tag. Keep examples short and self-contained.

## Touch Points

- Keywords list: `LANGUAGE_SPEC.md` §2.3 (39 keywords) — mirror in `docs/reference/keywords.md`.
- Operator precedence: §2.4 — mirror in `docs/reference/operator-precedence.md`.
- EBNF grammar: §16 — mirror in `docs/reference/grammar.md`.
- Attributes: §12 — mirror in `docs/reference/attributes.md`.
- Compiler diagnostics: §18 — mirror in `docs/reference/compiler-errors.md`.

## JIT Index Hints

```pwsh
# Jump to a section by number
rg -n "^## §?4\.4a" LANGUAGE_SPEC.md

# Find every TODO in spec-plans
rg -n "TODO|TBD|FIXME" docs/spec-plans

# Diff prompt vs full spec for a topic (manual cross-check)
rg -n "Shared<T>" docs/spec-plans

# Token-size sanity (rough — words × 1.3 ≈ tokens)
(Get-Content LANGUAGE_SPEC_PROMPT_CORE.md -Raw).Split().Count
(Get-Content LANGUAGE_SPEC_PROMPT_WEB3.md -Raw).Split().Count
```

## Common Gotchas

- **Renaming a section changes anchor links** in the rendered docs. If a section is referenced externally (other `docs/` files, GitHub issues), use a redirect note rather than a silent rename.
- **Prompt-mirror drift is the #1 risk.** Every spec PR must touch the relevant prompt-mirror file (`LANGUAGE_SPEC_PROMPT_CORE.md` and/or `LANGUAGE_SPEC_PROMPT_WEB3.md`) or explicitly justify why no update is needed.
- **`@fast_math`, all `f32`/`f64` math methods, actor primitives, and FFI are compile errors on-chain.** When changing on-chain semantics, sweep for these prohibitions in `docs/web3/` and `docs/stdlib/math.md`.
- **`Shared<T>` (§4.4a)** is new in v0.5.2 — when adding new types, check whether `Shared<T>` semantics need a paragraph too.

## Pre-PR Checks

```pwsh
# 1. Confirm Appendix D has a new entry for material changes
rg -n "## Appendix D" LANGUAGE_SPEC.md

# 2. Confirm the prompt mirror was edited if the spec was
git diff --name-only main...HEAD | findstr "spec-plans"

# 3. Sweep for cross-tree mirrors
rg -n "<your-changed-term>" docs
```
