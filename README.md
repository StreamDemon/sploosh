# Sploosh

**The AI-native programming language.** Rust-grade safety, Elixir-style actor concurrency, and native web3 targets — designed from the first keyword so that LLMs write correct code on the first try.

[![Rust CI](https://github.com/StreamDemon/sploosh/actions/workflows/rust.yml/badge.svg)](https://github.com/StreamDemon/sploosh/actions/workflows/rust.yml)
[![Prompt Budget](https://github.com/StreamDemon/sploosh/actions/workflows/prompt-budget.yml/badge.svg)](https://github.com/StreamDemon/sploosh/actions/workflows/prompt-budget.yml)
[![Spec](https://img.shields.io/badge/spec-v0.5.14--draft-orange)](docs/spec-plans/LANGUAGE_SPEC.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Why an AI-native language?

Every mainstream language was designed for humans; LLMs learn to imitate idiomatic human code as a side effect of training. Sploosh inverts that premise:

- **One canonical way to express every operation.** No syntactic synonyms, no style debates, no ambiguity for a model to guess wrong.
- **Vocabulary drawn from the most-trained tokens** across the top dozen languages — `fn`, `let`, `match`, `impl`, `spawn`. Nothing novel to memorize, for humans or models.
- **The whole language core fits in a system prompt.** The [prompt-sized spec mirror](docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md) is CI-enforced to stay under 5,600 tokens (`cl100k_base`), with a [web3 companion](docs/spec-plans/LANGUAGE_SPEC_PROMPT_WEB3.md) under 1,500. Paste it next to your task and the model has the *entire* language in context.
- **Diagnostics designed for agents.** Stable error codes, machine-applicable fixes, and NDJSON output built for tool-loop consumption, not just terminal reading.

The result is a language where "prompt → compilable program" is the primary workflow, not a party trick.

## What it looks like

Safety and concurrency without ceremony — no null, no exceptions, no shared mutable state, checked arithmetic everywhere:

```sploosh
actor Counter {
    state: i64,
    fn init(n: i64) -> Self { Counter { state: n } }
    pub fn inc(&mut self, n: i64) { self.state = self.state + n; }
    pub fn get(&self) -> i64 { self.state }
}

fn main() -> Result<(), AppError> {
    let c: Handle<Counter> = spawn Counter::init(0);
    send c.inc(5);           // fire-and-forget message
    let val = c.get();       // request/reply, blocks
    print("count = {val}");
    Ok(())
}
```

Errors are values, and pipelines propagate them per stage:

```sploosh
let report = input |> parse? |> validate? |> transform?;
```

The same language compiles to smart contracts. Storage layout is Solidity-compatible, reentrancy is guarded by default, and the compiler rejects anything non-deterministic on-chain:

```sploosh
onchain mod token {
    storage {
        balances: Map<Address, u256>,
        total_supply: u256,
    }

    pub fn transfer(to: Address, amount: u256) -> Result<(), TokenError> {
        let sender = ctx::caller();
        let bal = storage::get(&self.balances, sender)?;
        if bal < amount {
            return Err(TokenError::InsufficientBalance);
        }
        storage::set(&mut self.balances, sender, bal - amount);
        storage::set(&mut self.balances, to, storage::get(&self.balances, to)? + amount);
        emit Transfer { from: sender, to, amount };
        Ok(())
    }
}
```

**One source tree, four targets:** native (LLVM), WASM, EVM bytecode, and Solana SBF.

## Design principles

1. **One way to do everything** — zero ambiguity for humans and models alike.
2. **Familiar vocabulary only** — every keyword comes from the most-trained languages.
3. **Explicit over implicit** — no implicit conversions, hidden control flow, or operator overloading.
4. **Errors are values** — all fallible operations return `Result<T, E>`.
5. **Concurrency is structural** — supervised actors and message passing; no shared mutable state.
6. **Dual-target by design** — web2 and web3 from a single syntax, with target restrictions enforced at compile time.
7. **The spec fits in a prompt** — and CI fails the build if it stops fitting.

Read the full pitch in [VISION.md](VISION.md) and the reasoning behind every decision in the [design rationale](docs/rationale/why-sploosh-looks-this-way.md) and the spec's design-decision log.

## Project status

Sploosh is **spec-first and pre-1.0**. The language definition is complete through [`LANGUAGE_SPEC.md` v0.5.14-draft](docs/spec-plans/LANGUAGE_SPEC.md) — 18 sections plus a validated EBNF grammar — and the compiler bootstrap is underway in Rust:

| Crate | What it does |
|---|---|
| [`sploosh-lexer`](crates/sploosh-lexer) | Tokenizer for the full §2 surface (45 keywords: 36 reserved + 9 contextual) |
| [`sploosh-parser`](crates/sploosh-parser) | Recursive-descent parser targeting the §16 EBNF, corpus-tested against `tests/corpus/*.sp` |
| [`sploosh-ast`](crates/sploosh-ast) | Typed AST with attribute preservation and operator enums |

## Roadmap

The compiler roadmap lives in [issue #66](https://github.com/StreamDemon/sploosh/issues/66) with GitHub Milestones per phase. Phases map directly onto spec sections — the spec is authoritative, and each phase implements the sections listed.

| Phase | Spec | Status |
|---|---|---|
| 1. Frontend: lexer & parser | §2, §16 | 🟢 **Active** — [milestone 1](https://github.com/StreamDemon/sploosh/milestone/1) |
| 2. Name resolution & modules | §10 | [milestone 2](https://github.com/StreamDemon/sploosh/milestone/2) |
| 3. Type checking & inference | §3 | [milestone 3](https://github.com/StreamDemon/sploosh/milestone/3) |
| 4. Ownership & borrow checking | §4 | [milestone 4](https://github.com/StreamDemon/sploosh/milestone/4) |
| 5. Diagnostics (cross-cutting) | §18 | [milestone 5](https://github.com/StreamDemon/sploosh/milestone/5) |
| 6. Code generation: LLVM / WASM / EVM / SVM | §11, §13 | [milestone 6](https://github.com/StreamDemon/sploosh/milestone/6) |
| 7. Runtime, stdlib & actor concurrency | §7–§9, §13, §14 | [milestone 7](https://github.com/StreamDemon/sploosh/milestone/7) |
| 8. Tooling: `sploosh build\|test\|check`, LSP | §13.3, docs/tooling | [milestone 8](https://github.com/StreamDemon/sploosh/milestone/8) |

## Explore the language

The `docs/` tree is the language — a complete, internally consistent definition kept in sync with every change:

- **[Language specification](docs/spec-plans/LANGUAGE_SPEC.md)** — the authoritative reference
- **[Prompt-sized spec](docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md)** — the whole core, ready to paste into an LLM system prompt
- **[Guide](docs/guide/getting-started.md)** — tutorials from basic types to actors and async
- **[Examples](docs/examples/)** — hello world, CLI tool, REST API, actor chat server, token contract
- **[Web3 docs](docs/web3/onchain-overview.md)** — the on-chain module model, storage, events, cross-contract calls
- **[Migration guides](docs/migration/)** — coming from Rust, Elixir, Solidity, or TypeScript
- **[Stdlib reference](docs/stdlib/)** — per-module APIs with target availability

## Contributing

The spec is authoritative: when compiler behavior and the spec disagree, the compiler is wrong — or a spec amendment lands first. Every behavioral change updates the spec and its mirrors in the same PR.

- Start with [AGENTS.md](AGENTS.md) for conventions (it's written for AI agents and humans alike — this repo practices what the language preaches).
- Open parser work is tracked in [milestone 1](https://github.com/StreamDemon/sploosh/milestone/1); issues are scoped and labeled by effort.
- Language change proposals go through the [spec-change issue template](.github/ISSUE_TEMPLATE/spec_change.md).

Building for the toolchain requires stable Rust 1.91+:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Sponsoring

Sploosh is an independent open-source project exploring a question the whole industry is circling: **what does a programming language look like when AI agents are first-class authors?** Sponsorship funds compiler development toward the four-target build matrix and the first end-to-end "prompt → deployed program" milestone. If your work touches AI coding agents, developer tooling, or on-chain infrastructure, [sponsoring Sploosh](https://github.com/sponsors/StreamDemon) directly accelerates a public testbed for LLM-native language design.

## License

[MIT](LICENSE)
