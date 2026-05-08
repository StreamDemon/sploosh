# AGENTS.md — `docs/guide/`

Tutorial-style pages aimed at humans learning Sploosh. Prose-heavy, example-rich, opinionated.

## Identity

The "book" of Sploosh. Each file is a topic-focused tutorial. These pages **are not authoritative** — they explain what `LANGUAGE_SPEC.md` already decided.

## Files

```
getting-started.md          basic-types-and-variables.md
functions-and-control-flow.md
ownership-and-borrowing.md  structs-enums-and-traits.md
generics-and-advanced-types.md
closures-and-iterators.md   error-handling.md
pipe-operator.md            strings-and-formatting.md
actors-and-concurrency.md   async-await.md
modules-and-visibility.md
```

## Patterns & Conventions

- **Voice:** friendly, second-person ("you write"), pragmatic. Match `actors-and-concurrency.md` and `ownership-and-borrowing.md` for tone.
- **Examples must compile in a hypothetical world.** No spec-illegal code, even in motivational snippets.
- **Lead with motivation, not syntax.** A two-sentence "why this exists" before any example.
- **Cross-link the spec.** End each guide with `Spec reference: §<n>` for readers who want the formal rules.
- **No new language design.** If a guide implies behaviour the spec doesn't say, the guide is wrong.

## Touch Points

- `getting-started.md` is the canonical first read — keep it under 1,500 words and make it install→hello-world→next-step.
- `actors-and-concurrency.md` and `ownership-and-borrowing.md` are the longest guides; they're also the most spec-load-bearing. Sweep them when §4 (ownership) or §8 (concurrency) changes.
- `pipe-operator.md` codifies the `expr |> f?` precedence rule (`(expr |> f)?`) — must mirror §5.

## JIT Index Hints

```pwsh
# Find every guide that mentions a feature
rg -l "Shared<T>" docs/guide

# Find the spec-reference footers
rg -n "Spec reference" docs/guide

# Find guides missing a spec reference
rg -L "Spec reference" docs/guide
```

## Common Gotchas

- **On-chain examples in concurrency/async/IO guides are illegal.** Actors, async, fs/net/io are all compile errors inside `onchain`. If a guide example uses them, mark it native/wasm only.
- **`?` propagation requires the function to return `Result<_, _>` or `Option<_>`** — don't show `?` inside a `fn ... -> ()` example.
- **No `unsafe`.** Sploosh has no `unsafe` block; FFI is "safe wrappers around `extern "C"`" only. Guides that imply otherwise are spec-violating.

## Pre-PR Checks

```pwsh
# If you changed a guide, did the underlying spec also change?
git diff --name-only main...HEAD | findstr -r "guide spec-plans"

# All guides should have a Spec reference footer
rg -L "Spec reference" docs/guide
```
