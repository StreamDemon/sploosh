# Contributing to Sploosh

Thanks for helping build an AI-native programming language. Sploosh is
**spec-first**: the docs define the language; the compiler implements them.
When the two disagree, either the compiler is wrong or a spec amendment lands
first — never a silent behavior change.

## Before you start

1. Read [VISION.md](VISION.md) for product context and non-goals.
2. Read [AGENTS.md](AGENTS.md) for conventions, git rules, and the definition of done.
   Nearest-wins: subtrees under `docs/` and `crates/` have their own `AGENTS.md`.
3. Skim the authoritative spec:
   [`docs/spec-plans/LANGUAGE_SPEC.md`](docs/spec-plans/LANGUAGE_SPEC.md)
   (currently v0.5.14-draft).
4. Follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

| Kind | Where to start |
|------|----------------|
| Compiler / parser work | [Milestone 1](https://github.com/StreamDemon/sploosh/milestone/1) and labeled issues under `compiler/parser` |
| Spec amendments | [.github/ISSUE_TEMPLATE/spec_change.md](.github/ISSUE_TEMPLATE/spec_change.md) — open an issue before a large design change |
| Docs, guides, examples | `docs/` tree; keep mirrors in sync with the spec |
| Bugs | [Bug report template](.github/ISSUE_TEMPLATE/bug_report.md) |
| Questions | [GitHub Discussions](https://github.com/StreamDemon/sploosh/discussions) |

Issues are labeled by area (`compiler/parser`, `spec`, `web3`, `documentation`),
priority, and effort (`effort/small` …). Prefer an existing issue over opening a
duplicate; comment if you intend to take it.

## Development setup

Requires **stable Rust 1.91+**.

```bash
git clone https://github.com/StreamDemon/sploosh.git
cd sploosh

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

On Windows, prefer the Ubuntu Docker check so local validation matches GitHub
Actions:

```powershell
.\scripts\docker-check.ps1 -Build
```

Docs need no build step. Sploosh source fixtures use the `.sp` extension under
`tests/corpus/`.

## Pull request process

- **Never commit to `main`.** Open a PR from a topic branch.
- **Branch names:** `spec/<topic>`, `docs/<topic>`, `feature/<topic>`, `fix/<topic>`.
- **Commits:** short descriptive title (no `feat:` / `fix:` prefixes); body explains *why*.
  No AI footers, `Co-Authored-By` trailers, or "Generated with" lines in commits,
  PR bodies, or issues.
- **Fill the PR template** (`.github/pull_request_template.md`): Summary, Spec
  Sections Affected, Build Targets Tested, Test Plan.
- Keep the branch up to date with `main` via **rebase**, not merge commits of `main`.
- Address `cubic-dev-ai` review comments or refute them with reasoning against the spec.

### Definition of done

- [ ] `LANGUAGE_SPEC.md` updated if behavior changed.
- [ ] Mirror docs synced when affected (`LANGUAGE_SPEC_PROMPT_CORE.md`,
      `LANGUAGE_SPEC_PROMPT_WEB3.md`, `docs/reference/`, `docs/stdlib/`, etc.).
- [ ] Examples still type-check by inspection until compiler coverage exists;
      add or update corpus tests when parser behavior is involved.
- [ ] Appendix D changelog entry for material spec changes.
- [ ] PR template sections filled.
- [ ] CI green (Rust fmt/clippy/test and prompt-budget where applicable).

## Spec-first rule (non-negotiable)

Behavioral language changes update
`docs/spec-plans/LANGUAGE_SPEC.md` **in the same PR** as any compiler or
mirror-doc change. Stale or contradictory docs are bugs.

Prompt-sized mirrors are CI-enforced:

- Core ≤ 5,600 `cl100k_base` tokens — `docs/spec-plans/LANGUAGE_SPEC_PROMPT_CORE.md`
- Web3 ≤ 1,500 tokens — `docs/spec-plans/LANGUAGE_SPEC_PROMPT_WEB3.md`

Run locally:

```bash
pip install -r scripts/requirements.txt
python scripts/check_prompt_budget.py
```

## Security

Do not open public issues for vulnerabilities. See [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions are licensed under the
[MIT License](LICENSE) that covers this repository.
