# AGENTS.md — `crates/`

Compiler bootstrap crates live here.

## Identity

These crates implement Sploosh from the specification. The spec remains authoritative:
when implementation behavior and `docs/spec-plans/LANGUAGE_SPEC.md` disagree, fix the
implementation or land a spec amendment first.

## Layout

| Crate | Purpose |
|---|---|
| `sploosh-ast` | Shared source spans and AST nodes. |
| `sploosh-lexer` | UTF-8 lexer for §2 and §16.1 tokens. |
| `sploosh-parser` | Recursive-descent parser targeting §16. |

## Conventions

- Keep crates small and dependency-light during bootstrap.
- Preserve source spans on public syntax nodes and diagnostics.
- Add corpus tests for every grammar shape accepted by the parser.
- Use `.sp` files for Sploosh fixtures.
- Treat contextual keywords as parser decisions. The lexer classifies reserved
  keywords and leaves contextual spellings as identifiers.

## Checks

Run native checks during development. Run the Ubuntu Docker check before PRs on
Windows; it mirrors GitHub Actions more closely and avoids local Cargo
lock/cache issues seen with Windows build directories.

```pwsh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Ubuntu-parity local check
.\scripts\docker-check.ps1 -Build
```
