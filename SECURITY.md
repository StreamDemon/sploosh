# Security Policy

## Supported versions

Sploosh is **pre-1.0 and spec-first**. There is no shipped language runtime or
released compiler toolchain yet. Security handling covers:

| Surface | Status | Notes |
|---------|--------|--------|
| Language specification (`docs/`) | Active | Design-level issues (unsound rules, on-chain footguns) |
| Compiler bootstrap crates (`crates/`) | Active | Lexer, parser, AST and future pipeline stages |
| CI / repo automation | Active | Workflows, scripts, supply chain |
| Generated or deployed on-chain artifacts | N/A until backends ship | EVM / Solana targets are specified, not emitted |

When stable releases exist, this table will list supported version lines.

## What to report

Please report anything that could harm users, maintainers, or future deployers:

- Memory safety, injection, or sandbox-escape issues in compiler crates or tooling
- Spec rules that enable unsound programs, silent data loss, or undefined behavior
- On-chain design flaws in the documented model (reentrancy, storage layout,
  cross-contract calls, payable handling, privilege confusion)
- Secrets exposure, unsafe defaults in scripts or GitHub Actions
- Supply-chain issues (malicious dependencies, workflow privilege escalation)

**Out of scope for private report** (use a normal issue or discussion instead):

- Feature requests and roadmap questions
- Style nits and non-security docs typos
- Parser bugs that only reject valid programs or accept invalid ones **without**
  a security consequence (still valuable — file a public bug)

## How to report a vulnerability

**Do not open a public GitHub issue for security reports.**

1. Email **revenantpulse@gmail.com** with subject line:
   `[SECURITY] sploosh — short description`
2. Include:
   - Affected path (spec section, crate, workflow, or script)
   - Impact (who can exploit it, what they gain)
   - Reproduction steps or a minimal proof of concept
   - Whether you have a suggested fix
3. Optional: open a draft GitHub Security Advisory on
   [StreamDemon/sploosh](https://github.com/StreamDemon/sploosh) if you prefer
   coordinated disclosure through GitHub (private fork / advisory workflow).

You should receive an acknowledgment within **72 hours**. If you do not, ping
again or try the same address with a different subject prefix.

## Our commitments

- We will confirm receipt and give an initial severity assessment when possible.
- We will work with you on a fix and a disclosure timeline before any public
  write-up.
- We will credit reporters who want credit (and omit names for those who do not).
- We will not take legal action against good-faith research conducted without
  privacy violations, data destruction, or service disruption.

## Disclosure timeline (target)

| Step | Target |
|------|--------|
| Acknowledgment | ≤ 72 hours |
| Initial triage | ≤ 7 days |
| Fix or mitigation plan | ≤ 30 days for high/critical in shipped code; longer for pre-release design issues when redesign is required |
| Public disclosure | After a fix is available, or by mutual agreement |

These are targets, not SLAs. Pre-1.0 design issues may require a spec amendment
PR before any code change; that work still happens under private coordination
when impact is security-relevant.

## On-chain and dual-target notes

Sploosh's web3 surface is part of the language design:

- Reentrancy is **guarded by default**; `@reentrant` is opt-in
- Storage layout aims at Solidity compatibility
- `onchain mod` rejects non-deterministic and host IO surfaces at compile time
  (as specified)

Until code generation backends ship, treat on-chain examples in `docs/` as
**specification**, not audited production contracts. Do not deploy "Sploosh"
contracts from this repo today — there is no supported compiler release.

Security reviews of the **spec** (especially §11 and related web3 docs) are
welcome and valuable before backends land.

## Bug bounties

There is **no paid bug bounty** at this time. Sponsorships that fund compiler
and security work are welcome via
[GitHub Sponsors](https://github.com/sponsors/StreamDemon).

## Preferred languages

English. Clear technical writing beats perfect grammar.
