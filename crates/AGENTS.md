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
  variants, `actor`, `mod`, `use`, `const`, `type`, `trait` incl. generics,
  supertrait bounds, and `where` clauses (bounds parsed but not stored),
  `impl Type` and `impl Trait for Type` (trait ref recorded in the AST),
  `onchain mod`, `extern`), with §16 modifier placement enforced —
  `offchain`/`async` only on `fn` items, and `pub` rejected on `impl`,
  `actor`, `onchain`, and `extern` items; types incl. generics, references,
  arrays/slices, tuples, `fn`, and `dyn Trait<Args>` incl. associated-type
  bindings; expressions incl. calls, turbofish, field/index,
  unary/binary (incl. `/`; `..`/`..=` are non-associative — chained ranges
  are a parse error), assignment (targets validated per §16
  `assign_target`), `?`, `.await`, `as`, struct literals (with the
  §5.1 block-head restriction), `vec!` (square brackets required —
  `vec!(...)` is a parse error), and `if`/`else`; `match` (§5.2) with the
  full §16 pattern grammar — literals, `_`, `[ref] IDENT` bindings, paths,
  tuple/call/struct destructuring incl. `..` rest, or-patterns, and `if`
  guards; the scrutinee is under the §5.2 block-head restriction, expression
  bodies need their trailing comma and block bodies none, and a non-tail
  `match` statement needs its `;` like `if`/blocks; patterns are wired into
  `match` arms, `select` arms, `if let`, `while let`, `for`, and closure
  params, and the two spec-example forms §16 cannot derive
  (`return` arm bodies — match §5.2/§8.8 and select §8.6 alike — and `ref`
  field-pat shorthand) are rejected pending #89;
  `if let` (§5.4, else is a plain block only — no `else if` chains after an
  if let, though `if`'s else chains into `if` and `if let`), `while`,
  `while let`, `for` (tuple/struct destructuring, range and pipe iterables),
  and `loop` — conditions, iterables, and if-let/while-let scrutinees all
  parse under the block-head restriction (the scrutinee positions pending
  #89 item 3), `break`/`continue` take no value and have no labels (§16,
  Appendix D), and the loop-context check on break/continue is semantic (no
  §18 code yet), so the parser accepts them in any statement position;
  closures (§4.6) — inferred/typed/wildcard/pattern params (top-level
  or-pattern params must parenthesize: the bare form collides with the closing
  delimiter, #89 item 4b), zero-arg `||` (`|`/`||` in prefix position opens a
  closure, infix `||` stays Logical OR — #89 item 4a), `move` closures, and no
  return-type annotation per §16; `spawn` (§8.2) — operand is an
  unrestricted expression (a struct-literal operand parses: spawn is not a
  block-head position) that greedily binds trailing pipes (`spawn (x |> f)`,
  §16-ambiguous pending #89), `spawn async` (§8.9) with a raw block, `select`
  (§8.6) with match-style arms (`pattern "=" expr "=>"` — full patterns
  allowed, the `=` delimiter means no closure-param collision; `timeout(ms)`
  is an ordinary call syntactically), and `emit` statements (§11.1/§11.3)
  reusing field-init shorthand — statement-only (no expression production),
  the on-chain-only restriction is semantic, and emit is spanless per the
  Stmt convention; `send` followed by a closure opener, `spawn`, or `select`
  now opens a send-statement (the operand then fails the method-call check)
  since those can begin an expression per §2.7; unit `()` and tuple `(a, b)`
  expressions (`Ok(())` pervades the spec's examples; §16 has no explicit
  alternative — pending #89 item 5), with `(a)` still grouping;
  `|>` with §16
  `pipe_stage` stages (`callee [args] [?]` — a stage's trailing `?` wraps the
  accumulated pipe application per §5.7), including the `"(" closure ")"`
  stage form (parens in a stage exist only to wrap a closure — `x |> (v)` is a
  parse error); `let`, `return`,
  `break`, `continue`, `send` (statement-head rule per §2.7 — opens a
  send-statement only when the next token can begin an expression, and the
  operand must be a method call), and expression/tail statements.
- **Not yet implemented:** patterns in `let`
  bindings — `let` binds a single identifier only, no destructuring
  (`let (a, b) = ...` and `let Some(x) = ...` are rejected); `#[...]` compiler directives are not parsed
  in any position (`#[cfg(test)]`, `#[target(...)]`, `#[indexed]` all fail);
  `storage { }` blocks inside `onchain mod` are consumed but discarded, not
  stored in the AST; turbofish on a non-final pipe-stage segment is rejected
  with an explicit not-yet-implemented parse error; literal-overflow checking
  (a parse-time
  error per `docs/reference/grammar.md`) is deferred to semantic analysis,
  where literals gain types; generic parameters and
  `trait`/`impl` bodies are skipped, not stored in the AST; `let mut` / `&mut`
  mutability and the `send` keyword are parsed but not preserved; a block-like
  expression (`if`/`if let`/`match`/`while`/`for`/`loop`/`select`/`spawn
  async`/block) used as a non-tail statement needs a trailing `;` (loosening
  that is #62's call — that issue's set must decide whether the newly
  brace-final `select`/`spawn async` join it).
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
