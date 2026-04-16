# SPLOOSH Language Spec Review (against v0.4.4-draft)

> Independent review of the Sploosh spec across two axes: LLM-friendliness,
> and usability / performance / efficiency. Findings are severity-tagged;
> §7 is the prioritized action list.

---

## 1. Scope & Method

### 1.1 What was reviewed

| File | Version / state |
|---|---|
| `docs/spec-plans/LANGUAGE_SPEC.md` | v0.4.4-draft, 3,479 lines |
| `docs/spec-plans/LANGUAGE_SPEC_PROMPT.md` | v0.4.4, 225 lines |
| `docs/examples/{hello-world,token-contract,actor-chat-server,cli-tool,rest-api}.md` | current |
| `docs/guide/{getting-started,error-handling,actors-and-concurrency,generics-and-advanced-types}.md` | current |
| `docs/tooling/sploosh-toml.md` | current |
| `docs/stdlib/collections.md` | current (representative sample of stdlib docs) |
| `docs/runbooks/debugging-ownership-errors.md` | current |

### 1.2 What was NOT reviewed

- **The compiler.** There is no implementation yet; every claim about runtime behavior is taken from the spec at face value.
- **Benchmarks.** No numbers are asserted — only directional concerns grounded in known properties of the underlying technologies (LLVM, EVM, BEAM-style schedulers).
- **LLM hit rates.** The predictions in §3.6 are informed estimates, not measurements. A working compiler plus a test harness would be required to produce real numbers.
- **Cross-target ABI correctness.** Storage-layout and ABI claims in §11.1a / §11.4a are reviewed for consistency with the Solidity reference, not for binary equivalence.

### 1.3 Severity legend

| Severity | Meaning |
|---|---|
| **Blocker** | Design contradiction or correctness bug. Fix before the next milestone. |
| **High** | Adoption-blocking gap. Fix before v1.0. |
| **Medium** | Meaningful friction. Fix opportunistically. |
| **Low** | Polish. |
| **Nit** | Wording or formatting. |

---

## 2. Executive Summary

**LLM-friendliness.** The per-token vocabulary is excellent — almost everything parses as Rust to a Rust-trained model, which is the right statistical bet. The *combinations* are novel and not in training data (Rust syntax + Elixir actor keywords + Solidity on-chain surface), and several rules are subtle enough that LLMs will consistently miss them (`send` only on `&mut self`, handle drop ≠ actor death, on-chain float values-vs-methods split). There is one concrete contradiction in the spec (`none` vs `None`, §3.3) and several "one way to do it" violations that undermine the stated design principle.

**Usability.** The language spec is mature; the *tooling* spec is mostly TODOs. For a language whose primary differentiator is generation accuracy, the compiler-error format is the single highest-leverage artifact — and it's unspecified. Human learning curve compounds three languages' worth of concepts (Rust ownership, OTP supervision, Solidity semantics); the guide docs don't stage this.

**Performance.** The ceiling is Rust-class (LLVM backend, zero-cost iterators, float intrinsics, BEAM-style scheduler). The floor is lower than Rust's because there is no `Arc<T>`-equivalent for shared immutable data, forcing clone-or-actor-wrap patterns that add allocations or latency relative to idiomatic Rust. Orphaned actors (§8.2) are a memory leak by design.

**Efficiency.** Strong on message passing (zero-copy moves); weak on read-heavy shared state; under-characterized on binary size, startup cost, and compile time.

### Top 5 prioritized recommendations

| # | Severity | Recommendation |
|---|---|---|
| 1 | Blocker | Resolve the `none` vs `None` contradiction (§3.3). |
| 2 | Blocker | Decide the shared-immutable-data story: adopt `Arc<T>` or introduce `Shared<T>`; document the cost either way (§5.2). |
| 3 | Blocker | Specify the compiler-error format as a first-class design artifact (§4.1). |
| 4 | High | Add `handle.stop()` or equivalent — orphaned actors are a permanent leak (§5.2). |
| 5 | High | Fill in the tooling spec (lockfile, workspace, profiles, dev-deps) before adding more language features (§4.1). |

---

## 3. LLM-Friendliness Review

### 3.1 What works

- **Rust-flavored keywords and operators** (`fn`, `let`, `match`, `?`, `|>`, `->`, `=>`, `as`, `trait`, `impl`). Dense training data per token.
- **No `unsafe`, no raw pointers, no null, no exceptions.** Closes entire categories of LLM misuse.
- **Checked arithmetic by default.** LLMs don't reason about overflow; the escape hatches (`wrapping_add`, `@overflow(wrapping)`) are opt-in and explicit.
- **`Result<T, E>` + `?`.** One of the most LLM-friendly error patterns in modern languages; already well-trained.
- **ASCII-only identifiers, braces over indentation.** Maximal tokenizer accuracy.
- **Deterministic `select` arm ordering** (§8.6). Reproducible and unsurprising.
- **§17 design decisions log.** Evidence that these tradeoffs were considered explicitly, not by accident.

### 3.2 Stated-vs-actual mismatches

#### Token budget (§1 design principle #7)

§1 claims: *"The full language core is describable in under 4,000 tokens for LLM system prompts."*

Measured with `tiktoken` against both OpenAI encodings (`cl100k_base` = GPT-4 / Claude-family proxy; `o200k_base` = GPT-4o / newer models):

| Artifact | Words | Bytes | `cl100k_base` | `o200k_base` |
|---|---|---|---|---|
| `LANGUAGE_SPEC_PROMPT.md` | 1,771 | 13,660 | **4,077** | **4,086** |
| `LANGUAGE_SPEC.md` | 22,302 | 153,135 | 41,180 | 41,226 |

The PROMPT edition is **currently 2% over the stated 4,000-token budget** on both encodings. It is not at-budget; it is narrowly but factually over.

Anthropic / vendor-specific tokenizers may differ (the `Read` tool in this environment reported ~47,800 tokens for the full spec, suggesting an even more aggressive split than cl100k's 41,180). The relative picture holds across tokenizers: PROMPT edition is approximately at-or-slightly-over budget today, and each amendment is adding to it (v0.4.4 added ~30 lines to the PROMPT edition).

**Severity: Medium.** Either (a) tighten the claim to "approximately 4,000 tokens" (honest, unremarkable), or (b) prune the PROMPT edition below 4,000 and gate future amendments on a tokenizer check in CI. Option (b) is the principled one for an "AI-native" language — a budget that isn't enforced isn't a budget.

#### "One way to do everything" (§1 design principle #1)

The spec has several places with two interchangeable forms:

- **Pipe vs method chains on iterators** (§7.3). The spec explicitly says *"These are identical"* for `x.iter().filter(...)...` vs `x.iter() |> filter(...) |> ...`. Two syntactic forms with identical semantics — an LLM will mix them in the same chain.
- **String construction:** `"hello".into()`, `String::from("hello")`, `format("{}", "hello")` — three paths to an owned `String`. All valid. All equally idiomatic per the spec.
- **Two `extern` forms** (§4.9, §11.4a): `extern "C" { ... }` and `extern onchain mod X { ... }` share grammar prefix but have different calling conventions, safety models, and error types. §11.4a ends with an explicit warning table — codifying the confusion instead of eliminating it.

**Severity: Medium.** Each individually is minor; together they weaken the design principle.

#### "No novel tokens" (§1 design principle #2)

§1 claims *"Every keyword and operator is drawn from the top 12 most-trained languages. No novel tokens."*

Novel in the training distribution for this combination of syntax:
- `onchain` (keyword), `offchain` (keyword), `storage` (block keyword), `emit` (statement keyword) — Solidity has analogues but not in this syntactic shape.
- `extern onchain mod` — invented phrase; no precedent.
- `spawn Counter::init(0)` — the spawn-applied-to-a-constructor-call form is novel. Rust's `std::thread::spawn` takes a closure; Erlang's `spawn` takes an MFA; neither matches this shape.
- `send counter.method(args)` — prefix statement form for messaging is novel.

The design principle is defensible as a per-token claim ("no new *lexemes*") but false as a per-construct claim.

**Severity: Low.** The claim should be softened in §1 rather than the language changed.

### 3.3 Concrete bug: `none` vs `None` inconsistency — **Blocker**

**Evidence** (verified with `grep`):

| Location | Content |
|---|---|
| `LANGUAGE_SPEC.md:47` | `` `self` `Self` `true` `false` `none` `as` `` — `none` listed as a keyword |
| `LANGUAGE_SPEC.md:3280` | `literal = INT_LIT [ type_suffix ] \| FLOAT_LIT [ type_suffix ] \| STRING_LIT \| CHAR_LIT \| "true" \| "false" \| "none" ;` — `none` as a grammar literal |
| `LANGUAGE_SPEC.md:3010` | Prelude: `Option, Some, None` — capital `None` |
| Every code example in `docs/examples/` | Uses `None` or no Option at all; **zero** uses of lowercase `none`. |
| `docs/guide/generics-and-advanced-types.md:9`, `docs/guide/error-handling.md:11`, `docs/guide/actors-and-concurrency.md:44` | All use `None`. |

In other words: `none` is reserved in two definitional sites and used in zero practical sites. Meanwhile `None` is used everywhere but is technically just an identifier (the `Option::None` constructor exported from the prelude).

**Why this matters for LLMs.** Rust-trained models will emit `None` 100% of the time. If the lexer reserves `none` as a keyword, then `None` still works (it's an identifier that happens to resolve to the prelude constant) — but then the grammar's `literal = ... | "none"` production is dead code, and the keyword reservation serves no purpose.

**Recommendation:** Delete `none` from §2.3 keywords and from §16's `literal` production. `None` becomes the sole form, consistent with every example.

**Severity: Blocker** — spec self-contradiction; cheap fix.

### 3.4 Rule splits LLMs will consistently miss

These are the highest-frequency error sites for LLM code generation. Each is individually defensible; collectively they represent a cognitive load that undermines the accuracy pitch.

| # | Rule | Spec § | Expected failure mode |
|---|---|---|---|
| 1 | Actor `&mut self` methods **must** use owned params; `&self` methods **may** take references. | §8.2 | Rust training says "prefer `&str` over `String`"; LLMs will write `pub fn log(&mut self, msg: &str)` — compile error. |
| 2 | `send` is valid only on `&mut self` methods. `send handle.get()` on an `&self` method is a compile error. | §8.2 | LLMs will treat `send` as universal fire-and-forget; will emit `send counter.get()`. |
| 3 | Dropping a `Handle<T>` — even the last one — does **not** terminate the actor. | §8.2 | LLMs assume RAII cleanup; will design code that relies on handle-drop lifecycle. |
| 4 | `init` panic → actor is `DEAD` but `spawn` returned a live-looking handle. First call returns `Err(ActorError::Dead)` or silently drops. | §8.1a | LLMs will not write the dead-on-first-call handling path. |
| 5 | `?` (precedence 12) binds tighter than `\|>` (precedence 8). `x \|> f?` = `(f(x))?`. | §5.7 | LLMs will guess either way and be right half the time. |
| 6 | On-chain functions are **non-reentrant by default**. | §11.3, §11.3a | Solidity-trained models will reflexively add `nonReentrant` modifier patterns. |
| 7 | On-chain: `f32`/`f64` **values** are legal (`==`, `<`, field storage, arguments) but **method calls** (`.sqrt()`, `.abs()`, even classification) are a compile error. | §4.10, §12.3 | LLMs will read "floats allowed on-chain" and emit `x.abs()`. |
| 8 | `@reentrant` disables the guard for one function only; sibling `pub` functions are still guarded. | §11.3a | LLMs will assume contract-wide opt-out. |
| 9 | `@fast_math` is **not inherited** by callees. | §12.1 | LLMs will expect transitive effect. |
| 10 | `init` is **infallible by signature** — it returns `Self`, not `Result<Self, E>`, and cannot be `async`. | §8.1a | LLMs will want `fn init(args) -> Result<Self, InitError>`. |
| 11 | Integer literal default is `i64`; `.len()` returns `u64`. `items.len() as u64` and `vec.len() as i64` casts are pervasive. | §3.8 | LLMs will forget the cast, hitting type-mismatch errors. |

### 3.5 Combinations not in training data

Per-token, Sploosh parses as Rust. Per-construct, several shapes have no training precedent:

- `spawn Counter::init(0)` — `spawn` applied to a free function call, not a closure.
- `send counter.method(args)` as a statement-prefix form.
- `emit Transfer { ... }` as a statement-prefix form.
- `onchain mod X { storage { ... } ... }` — module with a `storage` block.
- `extern onchain mod X { pub fn ...; }` — signatures-only block with `pub` that is parsed but ignored.

Rust code never uses `actor`/`spawn`/`send` with this grammar; Elixir code never uses `struct`/`impl`/`Result`/`?`; Solidity code never uses `match`/`enum`/`trait`. An LLM generating Sploosh is **interpolating** three training distributions, not pattern-matching on one. Interpolation errors compound in longer programs.

### 3.6 Predicted first-try compile rate (frontier model, PROMPT edition in context)

These are calibrated guesses, not measurements. They would make a good correctness-tracking benchmark once a compiler exists.

| Task | Estimated first-try compile rate |
|---|---|
| Hello world; simple fn; `Result` + `?` | 90%+ |
| Iterator chains; pattern matching | 80–90% |
| Struct + trait + impl (no lifetimes) | 70–80% |
| Lifetime-annotated code | 50–70% (Rust-equivalent) |
| Single actor + `spawn` + `send` | 50–65% (novel combo) |
| Multi-actor + supervisor + `select` | 30–50% (rule-density cliff) |
| On-chain contract + storage + reentrancy + `chain::call` | 20–40% (most novel surface) |

**Implication.** Without active targeting of the rules in §3.4 through error messages, docs, or a "common mistakes" section in the PROMPT edition, Sploosh will land at *roughly Rust-equivalent* LLM-friendliness — good, but not the differentiator the positioning promises.

### 3.7 LLM-specific recommendations

| # | Severity | Recommendation | Where |
|---|---|---|---|
| L1 | Blocker | Delete `none` from §2.3 keywords and §16 `literal` production. | §2.3, §16 |
| L2 | High | Add a "Common LLM mistakes" appendix to `LANGUAGE_SPEC_PROMPT.md` covering the rules in §3.4 above. Negative examples are denser than positive ones; 300 tokens of this is worth 2,000 tokens of prose. | PROMPT edition |
| L3 | High | Collapse `iter()` + pipe + method-chain into **one** recommended form. Either remove pipe-for-iterators or remove method-chains-on-Iter. The other can still parse. | §7.3 |
| L4 | Medium | Rename `extern onchain mod` to eliminate the `extern "C"` visual collision. `contract interface X { ... }` or `onchain interface X { ... }` would break ties without raising cognitive load. | §11.4a |
| L5 | Medium | Soften §1 design principle #2 from "no novel tokens" to "no novel lexemes" — the claim is true per-token, false per-construct. | §1 |
| L6 | Medium | Put `u256` off-chain cost into the PROMPT edition. LLMs asked for a general-purpose counter will happily pick `u256`. | §3.1, `stdlib/math.md` |
| L7 | Low | Split PROMPT edition into `core.md` + `web3.md`. A REST-endpoint prompt doesn't need on-chain context. | PROMPT edition |
| L8 | Medium | Enforce the 4,000-token PROMPT-edition budget in CI (e.g., a `tiktoken` check on `cl100k_base` in a pre-commit or CI step), **or** tighten §1 principle #7 to match reality (current count: 4,077 tokens cl100k / 4,086 o200k). | PROMPT edition, §1 |

---

## 4. Usability Review (Human Developers)

### 4.1 Tooling gaps — **Blocker / High**

The language spec is detailed; the tooling spec is not.

| Area | Current state | Recommendation |
|---|---|---|
| `sploosh.toml` spec | 45 lines, trailing `<!-- TODO: Expand with workspace support, dev-dependencies, build profiles once implemented -->` | Spec these before adding more language features. Each is adoption-blocking. |
| Lockfile format | Unspecified | Define: TOML vs binary; hash algorithm; registry URL encoding; workspace-member handling. |
| Dependency resolution | Unspecified | Semver strictness; minimum-version selection vs latest-compatible; registry model; git/path deps. |
| Build profiles | Unspecified | dev / release / test / bench; opt-level; LTO; codegen-units; strip. |
| Compiler error format | Unspecified | **Highest-leverage artifact in the repo for the AI-native positioning.** Spec: stable error codes, machine-readable spans, suggested-fix blocks, diagnostic JSON mode. |
| LSP / editor integration | Not speced | Incremental compilation story; indexing model; inlay hints. |
| Formatter | Not speced | A strict, no-options formatter is the single biggest correctness multiplier for AI-generated code. |
| Linter | Not speced | clippy-equivalent with stable lint names. |
| Debugger | Not speced | DWARF emission; actor-aware stepping; mailbox introspection at breakpoint. |
| Test framework | `@test` attribute + `std::test` module name; no detail | Unit / integration layout; assertion macros (really intrinsics); property testing story; snapshot tests. |
| Actor tracing / observability | Not speced | BEAM's `:observer`/`:recon`/`:dbg` are load-bearing for Elixir's production success. Sploosh has nothing analogous. |

**Why this is Blocker / High.** The LLM-friendliness pitch *depends* on compiler errors being machine-actionable. Every other Rust-ecosystem-derived language (Swift, Kotlin via the KSP route, Move) has learned that errors-as-data is table stakes. Speccing the error format is a one-week document that pays off for the life of the project.

### 4.2 Learning curve

The pitch is "Rust safety + Elixir concurrency + web3." The curriculum required to be productive is:

1. Rust ownership, borrowing, lifetimes (weeks).
2. Rust trait system, generics, trait objects, coherence (weeks).
3. Rust error handling (`?`, `From`, `@error`) (days).
4. Elixir/OTP actor model, supervision trees, restart strategies (weeks).
5. Solidity semantics: gas, reentrancy, ABI encoding, storage slots (weeks).
6. Sploosh-specific overlay rules from §3.4 (hours per rule, many rules).

Each of 1–5 is a multi-week on-ramp in its home language; Sploosh asks the learner to take all of them at once.

**Recommendation (Medium).** Stage the guide. `docs/guide/` has 13 tutorials today but no ordering. Add a "learning path" table in `getting-started.md` that routes:
- Web backend developer → native-only subset (skip actors initially, skip web3).
- Elixir developer → actor chapters first, borrow checker second.
- Solidity developer → web3 chapters first, off-chain actors deferred.

### 4.3 Daily-driver friction

| Friction | Spec § | Severity |
|---|---|---|
| `"hello".into()` / `String::from("hello")` / `format("{}", "hello")` ceremony for an owned string | §9.5 | Medium |
| Mandatory `.clone()` on `Handle<T>` before passing | §8.2 | Low (matches Rust `Arc::clone`) |
| `items.len() as u64` / `vec.len() as i64` casts everywhere | §3.8 | Medium |
| Manual `Display` impls (only `Debug` is derivable) | §9.3, §12.2 | Medium — consider `@derive(Display)` with a default formatter |
| `format(...)` not `format!(...)` — muscle memory collision for every Rust user | §9.1 | Low |
| No `+` for strings | §9.5 | Low (principled) |
| No f-strings / template literals — every formatted string is a function call | §9.4 | Medium — the safety argument is weak; string interpolation with compile-time-checked placeholders has no more foot-guns than `format()` |

### 4.4 Documentation completeness

`docs/stdlib/collections.md` ends with `<!-- TODO: Add detailed API signatures and examples as stdlib is implemented -->`. Most stdlib docs follow the same pattern — a one-screen outline with a TODO.

That's fine for a spec-only project *provided* the policy is explicit: "stdlib docs are outline-quality until the compiler ships." Right now that policy is implicit. A reader following the CLAUDE.md guidance ("docs are the sole source of truth") will be misled.

**Recommendation (Low).** Add a one-line banner at the top of each TODO'd stdlib doc: *"This page is outline-quality; API signatures are indicative. Detailed specs land with the compiler milestone."* Removes the ambiguity without doing the work prematurely.

### 4.5 Usability recommendations

| # | Severity | Recommendation |
|---|---|---|
| U1 | Blocker | Spec the compiler error format (error codes, JSON mode, suggested-fix data). This is the single highest-leverage artifact for the AI-native positioning. |
| U2 | High | Flesh out `docs/tooling/sploosh-toml.md`: lockfile format, workspace support, dev-deps, build profiles, registry model. |
| U3 | High | Spec `std::test`, including integration-test layout and property-testing shape. |
| U4 | High | Spec the actor observability story (mailbox depth, restart history, live process list). |
| U5 | Medium | Add a staged learning path to `docs/guide/getting-started.md` with audience-based entry points. |
| U6 | Medium | Make `Display` derivable; the manual-only rule pays no safety dividend. |
| U7 | Low | Banner stdlib docs as outline-quality until compiler milestone. |

---

## 5. Performance Review

### 5.1 What's strong

| Asset | Why it matters |
|---|---|
| LLVM backend for native / wasm | Rust-class codegen; SIMD auto-vectorization available. |
| Float math as LLVM intrinsics (§4.10) | Constant folding, auto-vectorization, `sin`/`cos` fusion to `llvm.sincos`. Better than most languages out-of-the-box. |
| `@fast_math(flags)` with granular control (§12.1) | Avoids the all-or-nothing UB cliff of `-ffast-math`. |
| Checked arithmetic with explicit escape hatch (`wrapping_*`, `@overflow(wrapping)`) | Typical cost <5% on modern CPUs; safe default. |
| M:N work-stealing scheduler, lock-free bounded mailboxes (§8.11) | BEAM-proven architecture. |
| Zero-copy message passing (messages are moved, not cloned) | Large payloads are cheap across actors. |
| Lazy iterators (§7.2) | Same zero-cost-abstractions story as Rust. |
| Deterministic drop, no GC | No pause spikes; predictable latency. |

### 5.2 Real concerns

#### No `Arc<T>` / `Rc<T>` — **Blocker for the shared-immutable-data use case**

§4.4 states: *"No `Rc<T>` or `Arc<T>` in Sploosh. Use `Handle<T>` for sharing state across actors."*

This answer is correct for **mutable** shared state. For **immutable** shared state — lookup tables, parsed configs, ML weights, interned strings, read-only caches — the three available options are all bad:

1. **Clone it.** Expensive for large data; produces an allocation on every borrow boundary.
2. **Wrap it in an actor.** Every read is a message round-trip (mailbox enqueue + scheduler context switch + reply). Latency goes from nanoseconds to microseconds for what should be a pointer dereference.
3. **Pass `&T` locally.** Works inside one function; does not cross thread / actor / scope boundaries.

Rust uses `Arc<T>` heavily *precisely because* this class of problem is pervasive. The spec's answer obscures the cost rather than solving it.

**Recommendation:** Pick one of:

- **(a) Adopt `Arc<T>`.** Drop the "no Arc" principle. Accept one more concept in the surface area.
- **(b) Introduce `Shared<T>`** — an immutable-only refcounted type, with a compile-time rule that it cannot contain interior mutability. Simpler than `Arc<T>`; cannot create reference cycles because there's no `Weak<T>` or `Cell<T>` pairing.
- **(c) Document the cost** and explicitly leave this gap. Users will build their own. Worst outcome.

(b) is the most consistent with Sploosh's design philosophy (explicit, narrow primitive). Whatever you choose, decide in the next spec amendment — this is the largest unacknowledged cost in the language today.

**Severity: Blocker** for workloads that share immutable data (which is most non-trivial systems).

#### Orphaned actors (§8.2) — **High**

> *"An actor that is reachable from no live handle and has an empty mailbox is said to be **orphaned**; it continues running until the runtime shuts down."*

This is a memory leak. For long-running services with dynamic actor workloads (per-request actors, per-session actors, ephemeral workers), actors accumulate until process restart.

The spec acknowledges the tradeoff and defers `handle.stop()` to a future amendment. **Severity is High, not Blocker, only because** the workaround (supervisor-managed lifecycles) is available.

**Recommendation:** Add `handle.stop()` in the next amendment. The spec argues that "clean shutdown is the supervisor's job" — but the supervisor doesn't observe handle-drops either, so for actors that are not children of a supervisor (e.g., user-spawned task actors) there is no cleanup path at all.

#### Actor busy-across-await serializes read-heavy actors (§8.10) — **High**

While an actor `.await`s, its mailbox queues. Combined with "messages must be owned," a read-heavy actor (config cache, user lookup) with any I/O in its handler becomes a bottleneck.

This is standard BEAM-style actor semantics — but BEAM also has shared binary heap / refcounted binaries / ETS tables for exactly the read-heavy case. Sploosh has none of those.

**Recommendation:** This ties directly to the `Arc<T>`/`Shared<T>` decision above. A read-heavy cache should live behind `Shared<T>`, not inside an actor.

#### `u256` off-chain cost — **Medium**

§3.1 promotes `u256` to a primitive: *"Available on all targets. Literal suffix: `0u256`. Always uses checked arithmetic regardless of build mode."*

On EVM this is free (native word size). On native/wasm it's synthesized from four `u64`s — roughly 10–20× slower than `u64` for arithmetic, much more for `pow` / `isqrt`.

**Recommendation:** Warn explicitly in §3.1 and `stdlib/math.md`. LLMs will pick `u256` for general counters because the spec says it's universal.

#### Move-only messaging forces clones of read-heavy replies — **Medium**

A query actor returning `Vec<User>` to many callers clones the vector per reply. Elixir dodges this with refcounted binaries; Sploosh doesn't.

**Recommendation:** Again, ties to `Shared<T>`.

#### Missing performance knobs — **Medium**

| Knob | Rust equivalent | Impact |
|---|---|---|
| Layout control | `#[repr(C)]`, `#[repr(packed)]`, `#[repr(transparent)]` | FFI, memory density |
| Explicit SIMD | `std::simd`, `std::arch` | 4–16× speedup on numeric kernels |
| Custom allocators | `alloc_api`, arenas, pools | High-throughput servers |
| Compile-time evaluation | `const fn` | Removes runtime work; documentation-as-code |
| LTO / PGO switches | `cargo profile.release.lto`, `rustc -Cprofile-generate` / `-Cprofile-use` | 5–15% runtime improvement |
| Minimal stdlib subset | `#![no_std]` | Embedded + small-wasm targets |

None of these are in the v0.4.4 spec. For a "systems-grade" language targeting native + wasm, each is a known adoption blocker for its respective niche.

#### EVM codegen via Yul — **Medium**

§11.7 goes through Solidity Yul to EVM bytecode. This inherits Solidity's codegen efficiency; it will **not** beat hand-tuned Solidity assembly in gas-sensitive hotspots (DEX routers, math-heavy protocols).

**Recommendation:** Add an inline-EVM escape hatch as a deferred item. It's not v0.5.0 work, but it's v1.0 work for the gas-sensitive segment.

#### Compile speed unspecified — **Low**

Sploosh inherits Rust-style monomorphization (§3.9 explicitly says "zero-cost, monomorphized") and adds three backends (LLVM + EVM + SVM). Expect Rust-like compile times in generic-heavy codebases, amplified by multi-target CI.

**Recommendation:** Commit to an incremental-compilation target in the tooling spec. The language can win developer experience if builds are meaningfully faster than Rust's; it cannot afford to be slower.

### 5.3 Performance recommendations

| # | Severity | Recommendation |
|---|---|---|
| P1 | Blocker | Decide the shared-immutable-data story. Recommended: `Shared<T>` as an immutable-only refcounted primitive. |
| P2 | High | Add `handle.stop()` in the next amendment. |
| P3 | High | Re-examine actor-is-busy-across-await patterns once `Shared<T>` exists — many use cases move off actors. |
| P4 | Medium | Add explicit SIMD intrinsics or an `@vectorize` hint. |
| P5 | Medium | Spec `#[repr(C)]`, `#[repr(packed)]`, `#[repr(transparent)]`. Even if implementation is deferred, commit the surface. |
| P6 | Medium | Introduce `const fn`. Keeps the door open without requiring immediate broad expansion. |
| P7 | Medium | Add LTO / opt-level / codegen-units knobs to `sploosh.toml`'s build-profile section. |
| P8 | Medium | Add a `u256` off-chain cost warning to §3.1 and `stdlib/math.md`. |
| P9 | Low | Publish a compile-speed target for the compiler milestone. |

---

## 6. Efficiency Review (Resource Usage)

### 6.1 Memory

- **Default 1024-slot mailbox × actor count.** Configurable via `@mailbox(capacity: N)`. At 10K actors this is ~80 MB of mailbox state; at 1M (BEAM-scale fleet) it's 8 GB. Tunable but not guided — the spec should offer a rule-of-thumb sizing note.
- **No shared-immutable-data path** (see §5.2) pushes repeated allocation of read-mostly data.

### 6.2 Binary size

Unspecified. WASM deployment cares about every kilobyte; native embedded cares about ROM footprint. Whether the stdlib can be tree-shaken, whether monomorphization bloat matters, whether a `no_std`-like subset exists — none is addressed.

**Recommendation (Medium):** Add a size-conscious target profile and a tree-shaking policy. LTO + dead-code elimination should be default in release.

### 6.3 Startup cost

Spawning a supervision tree isn't characterized. BEAM-style systems spawn in microseconds per actor; a large tree can still take milliseconds to stabilize. For serverless / short-lived workloads, this matters.

**Recommendation (Low):** Commit to an order-of-magnitude target (e.g. "spawning 10K actors in a cold runtime completes in < 100 ms on a single core") in the runtime spec. It's not a contract today; setting expectations helps downstream planning.

### 6.4 WASM vs native cliff

Same source compiles to a single-threaded cooperative scheduler on wasm and an M:N preemptive scheduler on native. Actors that hit I/O on native parallelize; on wasm they serialize.

This isn't a bug — it's how browser wasm works — but it deserves explicit documentation. A user who profiles on native and deploys to wasm will see very different numbers.

**Recommendation (Medium):** Add a "WASM performance model" subsection to `runbooks/cross-target-builds.md`.

### 6.5 Efficiency recommendations

| # | Severity | Recommendation |
|---|---|---|
| E1 | Medium | Add tree-shaking / dead-code-elimination policy + a size-conscious build profile. |
| E2 | Medium | Document the WASM-vs-native performance model explicitly. |
| E3 | Low | Commit to a startup-cost target in the runtime spec. |

---

## 7. Prioritized Action List

Consolidated from §3, §4, §5, §6. Action IDs reference the source section for traceability.

| # | Severity | Area | Recommendation | Source ID | Spec section(s) |
|---|---|---|---|---|---|
| 1 | Blocker | LLM | Delete `none` from §2.3 keywords and §16 `literal` production. | L1 | §2.3, §16 |
| 2 | Blocker | Perf | Decide shared-immutable-data primitive. Recommended: introduce `Shared<T>` (immutable-only, refcounted). | P1 | §4.4 |
| 3 | Blocker | Usability | Spec the compiler error format (error codes, JSON mode, suggested-fix data). | U1 | New spec section |
| 4 | High | Perf | Add `handle.stop()` to fix the orphaned-actor leak. | P2 | §8.2 |
| 5 | High | Usability | Flesh out `sploosh.toml`: lockfile, workspace, dev-deps, build profiles. | U2 | `tooling/sploosh-toml.md` |
| 6 | High | Usability | Spec `std::test` (unit/integration layout, property testing). | U3 | `stdlib/test.md` |
| 7 | High | Usability | Spec actor observability (mailbox depth, restart history, live process list). | U4 | New runbook |
| 8 | High | LLM | Add "Common LLM mistakes" appendix to `LANGUAGE_SPEC_PROMPT.md` covering §3.4 rules. | L2 | PROMPT edition |
| 9 | High | LLM | Collapse `iter()` + pipe + method-chain into one recommended form. | L3 | §7.3 |
| 10 | Medium | Perf | Add explicit SIMD intrinsics or `@vectorize` hint. | P4 | New §4.11 |
| 11 | Medium | Perf | Spec `#[repr(C)]` / `#[repr(packed)]` / `#[repr(transparent)]`. | P5 | §3.1 or new |
| 12 | Medium | Perf | Introduce `const fn`. | P6 | §4.7 |
| 13 | Medium | Perf | Add LTO / opt-level / codegen-units to `sploosh.toml` build profiles. | P7 | `tooling/sploosh-toml.md` |
| 14 | Medium | Perf | Warn about `u256` off-chain cost. | P8 / L6 | §3.1, `stdlib/math.md` |
| 15 | Medium | LLM | Rename `extern onchain mod` to remove `extern "C"` collision. | L4 | §11.4a |
| 16 | Medium | LLM | Enforce 4K-token PROMPT budget in CI, or tighten §1 principle #7 (current: 4,077 cl100k / 4,086 o200k). | L8 | PROMPT edition, §1 |
| 17 | Medium | LLM | Soften §1 principle #2 from "no novel tokens" to "no novel lexemes". | L5 | §1 |
| 18 | Medium | Usability | Staged learning path in `getting-started.md`. | U5 | `docs/guide/getting-started.md` |
| 19 | Medium | Usability | Make `Display` derivable. | U6 | §9.3, §12.2 |
| 20 | Medium | Efficiency | Tree-shaking policy + size-conscious build profile. | E1 | `tooling/sploosh-toml.md` |
| 21 | Medium | Efficiency | Document the WASM-vs-native performance model. | E2 | `runbooks/cross-target-builds.md` |
| 22 | Low | LLM | Split PROMPT edition into `core.md` + `web3.md`. | L7 | PROMPT edition |
| 23 | Low | Usability | Banner TODO'd stdlib docs as outline-quality. | U7 | `docs/stdlib/*.md` |
| 24 | Low | Perf | Publish a compile-speed target. | P9 | Future runtime spec |
| 25 | Low | Efficiency | Commit to a startup-cost target. | E3 | §8.11 |

---

## 8. Out of Scope / Deferred

- **Implementation review.** No compiler exists; runtime behavior claims are taken from the spec.
- **Benchmark numbers.** Directional only; would require a working compiler.
- **Real LLM hit-rate measurements.** The §3.6 table is calibrated guesses until there's a test harness.
- **Cross-target ABI binary equivalence.** §11.1a storage layout is reviewed for source-level consistency with Solidity, not byte-exact equivalence.
- **Security review.** Reentrancy semantics are reviewed for internal consistency; cryptographic primitives and consensus-affecting determinism warrant a dedicated review pass.

---

## 9. References

| File | Purpose |
|---|---|
| `docs/spec-plans/LANGUAGE_SPEC.md` | Primary subject of this review |
| `docs/spec-plans/LANGUAGE_SPEC_PROMPT.md` | Token-budget evidence (§3.2) |
| `docs/examples/hello-world.md` | Smallest-program ergonomics |
| `docs/examples/token-contract.md` | On-chain pattern realism |
| `docs/examples/actor-chat-server.md` | Actor pattern realism |
| `docs/examples/cli-tool.md`, `rest-api.md` | Idiomatic-use samples |
| `docs/guide/getting-started.md` | Onboarding curve |
| `docs/guide/error-handling.md`, `actors-and-concurrency.md`, `generics-and-advanced-types.md` | `None` usage evidence (§3.3) |
| `docs/tooling/sploosh-toml.md` | Tooling-gap evidence (§4.1) |
| `docs/stdlib/collections.md` | Stdlib completeness evidence (§4.4) |
| `docs/runbooks/debugging-ownership-errors.md` | Doc-completeness evidence |

---

*This review was produced against v0.4.4-draft in April 2026. Section references
are to that version and may shift in later amendments. Re-run the review after
each major amendment — every severity tag has a half-life.*
