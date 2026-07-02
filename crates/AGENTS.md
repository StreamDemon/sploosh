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

## Implemented subset (bootstrap)

The parser covers a subset of §16, not the whole grammar. Treat anything outside
the accepted list as **not yet implemented** — add a corpus fixture when it lands.

- **Accepted:** every item form (`fn`, `struct`, `enum` incl. tuple/struct
  variants, `actor`, `mod`, `use`, `const`, `type`, `trait`, `impl`,
  `onchain mod`, `extern`); types incl. generics, references, arrays/slices,
  tuples, `fn`/`dyn`; expressions incl. calls, turbofish, field/index,
  unary/binary (incl. `/`), `?`, `.await`, `|>`, `as`, struct literals (with the
  §5.1 block-head restriction), `vec!`, and `if`/`else`; `let`, `return`,
  `break`, `continue`, `send`, and expression/tail statements.
- **Not yet implemented:** `match`, `while`, `for`, `loop`, `if let`, and
  closures (their keywords lex but have no parse production); `spawn`, `select`,
  and `emit` (reserved keywords with no expression/statement production yet, so
  §8 actor spawns and §11.5 event emission do not parse); patterns — `let` binds
  a single identifier only, no destructuring (`let (a, b) = ...` and
  `let Some(x) = ...` are rejected); `#[...]` compiler directives are not parsed
  in any position (`#[cfg(test)]`, `#[target(...)]`, `#[indexed]` all fail);
  `storage { }` blocks inside `onchain mod` are consumed but discarded, not
  stored in the AST; literal-overflow checking (a parse-time error per
  `docs/reference/grammar.md`) is deferred to semantic analysis, where
  literals gain types; generic parameters and
  `trait`/`impl` bodies are skipped, not stored in the AST; `let mut` / `&mut`
  mutability and the `send` keyword are parsed but not preserved; a block-like
  expression (`if`/block) used as a non-tail statement needs a trailing `;`.
- **Intentionally absent:** block comments — §2.2 keeps one way to comment
  (`//`, `///`).

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
