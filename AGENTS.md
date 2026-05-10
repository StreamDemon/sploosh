# AGENTS.md — Sploosh

> Hierarchical agent guide. Read this first, then the closest sub-folder `AGENTS.md`.

## Project Snapshot

- **Sploosh** — an AI-native programming language: Rust safety + Elixir concurrency + web3 dual-target (native/wasm/evm/svm).
- **Spec-only repository.** No compiler, no runtime, no source code. Every artifact lives under `docs/`.
- **Source of truth:** `docs/spec-plans/LANGUAGE_SPEC.md` (currently v0.5.2-draft).
- **Sub-trees have their own `AGENTS.md`** — read the nearest one to the file you're touching.

## Setup Commands

There is nothing to install or build. The repo is pure markdown.

```pwsh
# Clone
git clone https://github.com/StreamDemon/sploosh.git

# View the spec
code docs/spec-plans/LANGUAGE_SPEC.md
```

Optional (for ByteRover-aware agents):

```pwsh
brv status         # check local context tree
brv query "..."    # query curated knowledge
```

## Universal Conventions

- **Markdown only.** No source code yet. File extension for future Sploosh code is `.sp`.
- **Spec-first.** Every behavioural change lands in `docs/spec-plans/LANGUAGE_SPEC.md` first; mirrors elsewhere update in the same PR.
- **Wording style:** terse, declarative, present tense. Match the existing voice in `LANGUAGE_SPEC.md`.
- **Examples in spec snippets** use the `sploosh` code-fence language tag.
- **Section numbering:** the spec uses `§<num>`; cite as `§4.4a`, `§11.4a`, etc.

## Git & PR Rules (sacred)

- **Never commit to `main`.** It is branch-protected. All work goes through PRs.
- **Never `--force-push main`**, never `--no-verify`, never `--no-gpg-sign`.
- **Branch naming:** `spec/<topic>`, `docs/<topic>`, `feature/<topic>`, `fix/<topic>`.
- **Commit messages:** short descriptive title (no `feat:` / `fix:` prefixes), body explains *why*, sign with `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>` when authored by Claude Code.
- **No `--amend` on published commits.** Make new commits if a hook fails.
- **PR template:** `.github/pull_request_template.md` — fill `Summary`, `Spec Sections Affected`, `Build Targets Tested`, `Test Plan`.
- **Reviewer:** `cubic-dev-ai` bot. Validate every comment against the spec before acting; reply with the fix SHA or an explanation if invalid.

## Documentation = Language

Sploosh has no compiler, no Stack Overflow, no legacy code. The `docs/` tree IS the language. Stale or contradictory docs are bugs.

When you change `LANGUAGE_SPEC.md`, also check (and update if affected):

| Mirror | Path |
|---|---|
| Prompt-sized reference (core) | `docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md` |
| Prompt-sized reference (web3) | `docs/spec-plans/LANGUAGE_SPEC_PROMPT_WEB3.md` |
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

# Find all attribute references
rg -n "@(test|derive|inline|error|payable|reentrant|supervisor|mailbox|overflow|fast_math)\b" docs

# Find all on-chain compile-error rules
rg -n "compile error.*onchain|onchain.*compile error" docs

# List all spec sections defined in LANGUAGE_SPEC.md
rg -n "^## " docs/spec-plans/LANGUAGE_SPEC.md
```

## Definition of Done (any PR)

- [ ] `LANGUAGE_SPEC.md` updated if behaviour changed.
- [ ] All mirror docs synced (`LANGUAGE_SPEC_PROMPT_CORE.md`, `LANGUAGE_SPEC_PROMPT_WEB3.md`, `docs/reference/`, `docs/stdlib/`, etc.).
- [ ] Examples still type-check by inspection (no compiler yet — read carefully).
- [ ] Appendix D changelog entry added for material spec changes.
- [ ] PR template sections all filled.
- [ ] Branch is up to date with `main` (rebase, don't merge `main` into branch).
- [ ] `cubic-dev-ai` comments addressed or refuted with reasoning.

## Related Files

- `VISION.md` — product context, audience, design philosophy.
- `CLAUDE.md` — local-only project notes (gitignored).
- `.github/pull_request_template.md` — PR scaffold.
- `.github/ISSUE_TEMPLATE/spec_change.md` — proposing language changes.
