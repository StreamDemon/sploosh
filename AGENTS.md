# AGENTS.md — Sploosh

> Hierarchical agent guide. Read this first, then the closest sub-folder `AGENTS.md`.

## Project Snapshot

- **Sploosh** — an AI-native programming language: Rust safety + Elixir concurrency + web3 dual-target (native/wasm/evm/svm).
- **Compiler bootstrap has started.** The language remains spec-first, with early Rust crates under `crates/`.
- **Source of truth:** `docs/spec-plans/LANGUAGE_SPEC.md` (currently v0.5.14-draft).
- **Sub-trees have their own `AGENTS.md`** — read the nearest one to the file you're touching.

## Setup Commands

Docs need no build step. Compiler crates use Cargo. On Windows, prefer the
Ubuntu Docker check before PRs so local validation matches GitHub Actions and
avoids Windows Cargo lock/cache quirks.

```pwsh
# Clone
git clone https://github.com/StreamDemon/sploosh.git

# View the spec
code docs/spec-plans/LANGUAGE_SPEC.md

# Check compiler crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Check compiler crates in Ubuntu Docker (CI parity / Windows-stable path)
.\scripts\docker-check.ps1 -Build
```

Optional (for ByteRover-aware agents):

```pwsh
brv status         # check local context tree
brv query "..."    # query curated knowledge
```

## Universal Conventions

- **Docs plus Rust compiler crates.** Language docs live under `docs/`; compiler bootstrap crates live under `crates/`. Sploosh source fixtures use `.sp`.
- **Spec-first.** Every behavioural change lands in `docs/spec-plans/LANGUAGE_SPEC.md` first; mirrors elsewhere update in the same PR.
- **Wording style:** terse, declarative, present tense. Match the existing voice in `LANGUAGE_SPEC.md`.
- **Examples in spec snippets** use the `sploosh` code-fence language tag.
- **Section numbering:** the spec uses `§<num>`; cite as `§4.4a`, `§11.4a`, etc.

## Git & PR Rules (sacred)

- **Never commit to `main`.** It is branch-protected. All work goes through PRs.
- **Never `--force-push main`**, never `--no-verify`, never `--no-gpg-sign`.
- **Branch naming:** `spec/<topic>`, `docs/<topic>`, `feature/<topic>`, `fix/<topic>`.
- **Commit messages:** short descriptive title (no `feat:` / `fix:` prefixes), body explains *why*. **No AI footers anywhere** (maintainer rule, 2026-07-02): no `Co-Authored-By` trailers, no `Claude-Session:` lines, no "Generated with" lines — in commits, PR bodies, or issues.
- **No `--amend` on published commits.** Make new commits if a hook fails.
- **PR template:** `.github/pull_request_template.md` — fill `Summary`, `Spec Sections Affected`, `Build Targets Tested`, `Test Plan`.
- **Reviewer:** `cubic-dev-ai` bot. Validate every comment against the spec before acting; reply with the fix SHA or an explanation if invalid.

## Documentation = Language

Sploosh has no shipped compiler, no Stack Overflow, no legacy code. The `docs/` tree remains the language authority; compiler crates must follow it. Stale or contradictory docs are bugs.

When you change `LANGUAGE_SPEC.md`, also check (and update if affected):

| Mirror | Path |
|---|---|
| Prompt-sized reference (core, ≤5,600 cl100k tokens) | `docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md` |
| Prompt-sized reference (web3, ≤1,500 cl100k tokens) | `docs/spec-plans/LANGUAGE_SPEC_PROMPT_WEB3.md` |
| Project context (local) | `CLAUDE.md` (gitignored) |
| Reference docs | `docs/reference/{grammar,keywords,attributes,operator-precedence,type-conversion-rules,compiler-errors}.md` |
| Stdlib | `docs/stdlib/*.md` |
| Guides / web3 / migration / examples / runbooks | `docs/{guide,web3,migration,examples,runbooks}/*.md` |
| Tooling | `docs/tooling/*.md` |

If you spot drift outside the scope of your task, **flag it in the PR description** even if you don't fix it.

## Security & Secrets

- No code → no runtime secrets. Do not invent `.env` files.
- Never commit personal tokens or API keys. The repo is public.
- Issue templates and PR template under `.github/` are public.

## JIT Index — Where things live

| Topic | Path | Sub-AGENTS.md |
|---|---|---|
| Compiler crates | `crates/` | `crates/AGENTS.md` |
| Spec + prompt + review | `docs/spec-plans/` | `docs/spec-plans/AGENTS.md` |
| Reference (grammar, keywords, attrs) | `docs/reference/` | `docs/reference/AGENTS.md` |
| Stdlib API pages | `docs/stdlib/` | `docs/stdlib/AGENTS.md` |
| Guide tutorials | `docs/guide/` | `docs/guide/AGENTS.md` |
| Web3 / on-chain | `docs/web3/` | `docs/web3/AGENTS.md` |
| Migration guides | `docs/migration/` | `docs/migration/AGENTS.md` |
| Examples | `docs/examples/` | `docs/examples/AGENTS.md` |
| Runbooks | `docs/runbooks/` | `docs/runbooks/AGENTS.md` |
| Tooling docs | `docs/tooling/` | `docs/tooling/AGENTS.md` |
| GitHub templates | `.github/` | — |

### Quick-find commands (PowerShell-friendly, ripgrep preferred)

```pwsh
# Find a spec section (note: spec uses ### for sub-sections like 4.4a, ## for top-level)
rg -n "^### 4\.4a" docs/spec-plans/LANGUAGE_SPEC.md

# Find every mention of a keyword across docs
rg -n "Shared<T>" docs

# Find all attribute / directive references
rg -n "@(test|property|derive|inline|error|payable|reentrant|supervisor|mailbox|overflow|fast_math)\b|#\[(target|gas_limit|indexed|cfg)\b" docs

# Find all on-chain compile-error rules
rg -n "compile error.*onchain|onchain.*compile error" docs

# List all spec sections defined in LANGUAGE_SPEC.md
rg -n "^## " docs/spec-plans/LANGUAGE_SPEC.md
```

## Definition of Done (any PR)

- [ ] `LANGUAGE_SPEC.md` updated if behaviour changed.
- [ ] All mirror docs synced (`LANGUAGE_SPEC_PROMPT_CORE.md`, `LANGUAGE_SPEC_PROMPT_WEB3.md`, `docs/reference/`, `docs/stdlib/`, etc.).
- [ ] Examples still type-check by inspection until compiler coverage exists; add/update corpus tests when parser behavior is involved.
- [ ] Appendix D changelog entry added for material spec changes.
- [ ] PR template sections all filled.
- [ ] Branch is up to date with `main` (rebase, don't merge `main` into branch).
- [ ] `cubic-dev-ai` comments addressed or refuted with reasoning.

## Related Files

- `VISION.md` — product context, audience, design philosophy.
- `CLAUDE.md` — local-only project notes (gitignored).
- `.github/pull_request_template.md` — PR scaffold.
- `.github/ISSUE_TEMPLATE/spec_change.md` — proposing language changes.
