# AGENTS.md — `docs/stdlib/`

Per-module standard library reference. One markdown file per module. Each module documents its target availability (native / wasm / evm / svm).

## Identity

Stdlib pages describe **public API surface**, not implementation. They tell users which functions exist, their signatures, and where they are (or aren't) available.

## Files

```
actor.md       chain.md         collections.md   crypto.md   db.md
env.md         fs.md            io.md            json.md     log.md
math.md        net.md           test.md          time.md     web.md
```

## Patterns & Conventions

- **Each page declares target availability up front.** The standard form is the `**Available targets:** ...` prose line (e.g. `**Available targets:** native, wasm`). The ✅/❌ matrix form (`**Targets:** native ✅ · wasm ✅ · evm ❌ · svm ❌`) is acceptable for pages with complex availability caveats (`actor.md`, `test.md`).
- **Signatures are full Sploosh fn signatures** — explicit return types, fully annotated.
- **On-chain restrictions are first-class.** Modules that are unavailable on-chain say so in the first paragraph (`fs`, `net`, `io`, `db`, `web`, `env`, `log`, `time` — the latter two forbidden as of v0.5.12 — plus `test`, `actor`, and `math` float methods).
- **Numeric/math semantics:** `math.md` is the canonical location for float-method-on-chain restrictions; cross-link from the spec rather than duplicating prose.
- **Errors are typed.** Each fallible function returns `Result<T, ModuleError>` with the error enum defined inline or in a Types section.

## Touch Points

- `chain.md` — `chain::call`, the cross-contract call API. Cross-references `docs/web3/cross-contract-calls.md`.
- `math.md` — the longest stdlib page; floats vs ints, `@fast_math`, float constants. Cross-references spec §4.10 and on-chain rules.
- `collections.md` — `Vec<T>`, `Map<K,V>`, `Set<T>`, `Channel<T>`. (`Box<T>` is a language-level type — spec §4.4 — not a collections page topic.) The on-chain storage layout for these (§11.1a) lives in `docs/web3/storage-and-state.md`, not here.
- `actor.md` — `Handle<T>` introspection, `std::actor::observe`, supervisor restart history. Mirrors spec §8.12; second-largest page in the directory.
- `test.md` — `@test` attribute, assertion API. Companion to `docs/runbooks/testing-strategies.md`.

## JIT Index Hints

```pwsh
# Find every stdlib function in a module
rg -n "^pub fn " docs/stdlib/math.md

# Find on-chain prohibitions
rg -n "compile error|onchain|on-chain" docs/stdlib

# Cross-check a function against its usage in the guide
rg -n "fs::read" docs
```

## Common Gotchas

- **Don't add a function to a stdlib page without spec backing.** Stdlib additions are spec-level — they need a §13 (prelude/intrinsics) or §13-style amendment.
- **Target availability is binding** — if `math` says `evm ❌` for float methods, that's a compile error, not a runtime fail. Wording must reflect that.
- **`f32`/`f64` math methods are compile-error on-chain even when deterministic** (§4.10 / `math.md`). Float *values* are still allowed; only method calls are rejected.

## Pre-PR Checks

```pwsh
# If you touched a stdlib page, confirm spec & guide updates
git diff --name-only main...HEAD | findstr -r "stdlib spec-plans guide"

# Pages missing a target-availability line (either accepted form)
rg --files-without-match '\*\*(Available targets|Targets):\*\*' -g '!AGENTS.md' docs/stdlib
```
