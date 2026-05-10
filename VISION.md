# Sploosh — Product Vision

## What This Is

Sploosh is an AI-native programming language designed for LLMs to generate accurately on the first try. It blends Rust-level memory safety, Elixir-style actor concurrency, and dual-target compilation to web2 (native + WASM) and web3 (EVM + Solana SBF). The repository today is **spec-only** — the language definition, its prompt-sized mirror, and supporting documentation. The compiler comes next.

The thesis: today's programming languages were designed for humans, and LLMs have to imitate idiomatic human code as a side effect of training. Sploosh inverts that — it picks syntax and semantics from the **most-trained** vocabulary across the top dozen languages, with **one canonical way** to express every operation, so LLM completions converge on correct code with minimal context.

---

## Target Users

### Primary Users

- **AI coding agents and the developers who direct them.** People who write code primarily by prompting Claude, GPT, Gemini, etc., and want a target language where the model is right the first time.
- **Systems-grade application authors.** Teams building services, tools, and on-chain protocols who need Rust-class safety and performance without Rust's learning curve.
- **Web3 protocol authors.** Developers shipping to EVM and Solana from a single codebase, with Solidity-compatible storage layout and a built-in reentrancy guard.

### Secondary Users

- **Language designers and researchers** evaluating an "LLM-native" approach to language design.
- **Educators** looking for a language where the rules fit in a system prompt.

---

## Core Use Cases

1. **Prompt → working program in one shot.** A developer pastes the v0.5 prompt-sized spec into an LLM, asks for a feature, and gets compilable code without manual repair.
2. **Single source, four targets.** Author once, build to native, WASM, EVM bytecode, and Solana SBF.
3. **Actor systems without shared-state bugs.** Use Elixir-style supervised actors with Rust-style ownership inside each actor; no `Rc`/`Arc`, no shared mutable state.
4. **On-chain code with off-chain ergonomics.** Write contracts with the same syntax as native code, with the compiler enforcing on-chain prohibitions (no float math, no fs/net/io, no actors).
5. **Drop-in for Solidity authors.** Storage layout is Solidity-compatible by design; reentrancy is guarded by default; `extern onchain mod` + `chain::call` replaces hand-rolled ABI scaffolding.

---

## Design Philosophy

### Guiding Principles (from spec §1)

1. **One way to do everything.** Every operation has exactly one syntactic form. Zero ambiguity for both humans and models.
2. **Familiar vocabulary only.** Every keyword and operator is drawn from the top 12 most-trained languages. No novel tokens.
3. **Explicit over implicit.** No implicit conversions, no hidden control flow, no operator overloading.
4. **Errors are values.** All fallible operations return `Result<T, E>`. No exceptions, no panics in safe code.
5. **Concurrency is structural.** Actor-based isolation with message passing. No shared mutable state.
6. **Dual-target by design.** Single source compiles to native (LLVM), WASM (web2), and on-chain bytecode (web3).
7. **Spec fits in a prompt.** The language core stays prompt-loadable alongside real task context (~4K `cl100k_base` tokens as a soft target).

### Authorship Philosophy

- **Documentation is the language.** With no compiler yet, the `docs/` tree is the sole source of truth. Stale docs are language bugs.
- **Spec-first PRs.** Every behavioural change updates `LANGUAGE_SPEC.md` in the same PR — never a behaviour change without a spec change.
- **Conservative additions.** The spec resists features. Adding one means cutting another, or splitting the prompt-sized artifact.

---

## Domain Terminology

| Term | Definition |
|------|------------|
| **`onchain mod`** | A module compiled for blockchain targets (EVM/SVM). Subject to on-chain prohibitions: no floats math methods, no actors, no fs/net/io/db/web/env. |
| **Actor** | An isolated unit of concurrent execution with private state and a message mailbox. The only way to express mutable shared state. |
| **`Handle<T>`** | A `Clone + Send` reference to a running actor. Sends are fire-and-forget; request/reply blocks. |
| **`Shared<T>`** (§4.4a) | An immutable refcounted primitive. The only opt-in for shared immutable data without channelling through an actor. |
| **`@payable` / `@reentrant`** | On-chain function attributes. `@payable` opts into receiving native value; `@reentrant` opts a function out of the per-contract reentrancy guard. |
| **Reentrancy guard** | Runtime per-contract bool flag set on entry to non-`@reentrant` `pub` on-chain functions. Distinct from actor `SelfCall`. |
| **`chain::call`** | The cross-contract call API. Lowers to EVM `CALL` or Solana CPI. Returns `Result<T, ChainError>`. |
| **Pipe `\|>`** | Left-to-right pipeline operator. `expr \|> f` is `f(expr)`. `expr \|> f?` parses as `(expr \|> f)?`. |
| **Dual-target** | Sploosh's commitment to compile a single source to native, WASM, EVM, and SVM. |
| **Spec mirror** | A doc page that restates a part of `LANGUAGE_SPEC.md` for ergonomics. Mirrors must always agree with the spec. |

---

## Technical Boundaries

### What Sploosh IS

- A **safety-first** language: no `null`, no exceptions, no `unsafe`, no implicit conversions, no operator overloading, no shared mutable state outside actors.
- A **single-syntax dual-target** language: same code, four backends, with the compiler enforcing target-specific restrictions at compile time.
- A **prompt-loadable** language: the entire core fits in a system prompt alongside real task context.
- An **on-chain-aware** language: storage layout is Solidity-compatible, reentrancy is guarded by default, gas/compute-units are first-class.

### What Sploosh is NOT (Non-Goals)

- **Not a Rust replacement for kernel hacking.** No `unsafe`, no raw pointers, no inline assembly. FFI exists but only via safe wrappers around `extern "C"`.
- **Not a Lisp.** Macros, reader macros, and arbitrary metaprogramming are out. Attributes are the only metaprogramming surface.
- **Not a dynamic language.** No reflection, no eval, no runtime type information beyond what `dyn Trait` requires.
- **Not human-first.** Where ergonomics conflict with LLM accuracy, LLM accuracy wins. (Block comments were dropped because there are two ways to comment in most languages.)
- **Not a feature-rich language.** Adding a feature requires removing another, or splitting the prompt-sized artifact. The default answer to "can we add X?" is "no, unless you delete Y."

---

## Success Metrics

How we know if Sploosh is working:

1. **LLM first-shot accuracy.** Given the prompt-sized spec and a real task, frontier models produce compilable Sploosh code on the first attempt at significantly higher rates than for any incumbent language.
2. **Cross-target build matrix.** A non-trivial program (actor server + onchain module + WASM client) builds clean across all four targets from one source tree.
3. **Spec stability.** The spec converges on a 1.0 surface that fits the prompt budget and stays there.
4. **Solidity migration paths.** Existing ERC-20/ERC-721/governance contracts can be ported to Sploosh `onchain mod` form without changing storage layouts.

---

## Future Considerations

Things we might add later (not now):

- **Solana-target storage layout amendment.** SVM account-based storage rules are deferred to a future version.
- **Delegatecall.** Excluded from v0.4.x; revisit when the security story is clearer.
- **A real package registry.** `package-management.md` is forward-looking; an actual registry comes after the compiler.
- **An LSP.** Editor integration is aspirational until the compiler ships.
- **Effect tracking / capabilities** beyond on-chain prohibitions. Possible but not in scope for 1.0.

---

## Related Documentation

- `AGENTS.md` — root agent guide; nearest-wins hierarchy across `docs/`.
- `CLAUDE.md` — local-only project notes (gitignored).
- `docs/spec-plans/LANGUAGE_SPEC.md` — authoritative spec (v0.5.2-draft).
- `docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md` — prompt-sized mirror (language core).
- `docs/spec-plans/LANGUAGE_SPEC_PROMPT_WEB3.md` — prompt-sized mirror (§11 on-chain surface).
- `.github/pull_request_template.md` — PR scaffold.
- `.github/ISSUE_TEMPLATE/spec_change.md` — proposing language changes.
