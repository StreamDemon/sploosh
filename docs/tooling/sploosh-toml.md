# sploosh.toml

> Project manifest format. Authoritative schema lives in `LANGUAGE_SPEC.md`
> §14.1–§14.4; this page mirrors that surface for tool-side reference.

## Single-Package Example

```toml
[project]
name = "my-app"
version = "0.1.0"
edition = "0.5"
description = "An example Sploosh application"
license = "Apache-2.0"

[dependencies]
sploosh_web   = "0.3"
sploosh_chain = { version = "0.2", features = ["evm"] }
sploosh_db    = { version = "0.2", optional = true }

[dev-dependencies]
sploosh_test_utils = "0.1"

[features]
default = ["json"]
json = []
postgres = ["dep:sploosh_db"]
analytics = ["sploosh_db/metrics"]

[target.wasm.dependencies]
sploosh_web = { version = "0.3", default-features = false, features = ["client"] }

[targets]
default = "native"
contracts = ["evm", "svm"]

[profile.release]
opt-level = 3
lto = "thin"
strip = "debuginfo"

[runtime]
threads = 8
mailbox_default_capacity = 2048
```

## Workspace Example

```toml
# Root sploosh.toml — no [project], this is a workspace root
[workspace]
members = ["crates/*", "contracts/token"]
exclude = ["crates/scratch"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "0.5"
license = "Apache-2.0"

[workspace.dependencies]
sploosh_web   = "0.3"
sploosh_chain = "0.2"
```

```toml
# crates/api/sploosh.toml — member inheriting workspace defaults
[project]
name = "api"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
sploosh_web.workspace = true

[target.wasm.dependencies]
sploosh_chain = { workspace = true, features = ["client"] }
```

## Sections

### `[project]`

| Field | Required | Type | Notes |
|---|---|---|---|
| `name` | yes | string | ASCII identifier chars + `-`/`_`; cannot start with a digit. |
| `version` | yes | string | SemVer 2.0. |
| `edition` | yes | string | Sploosh language edition (`"0.5"`). Tracks released minor versions. Not a year. |
| `description` | no | string | One-line summary. |
| `license` | no | string | SPDX expression. |
| `authors` | no | list | `"Name <email>"` entries. |
| `repository` | no | string | Source URL. |

Unknown fields are a hard error — the manifest is a contract, not a hint
surface.

### `[dependencies]` / `[dev-dependencies]` / `[build-dependencies]`

All three share the same shape. `[dev-dependencies]` is test-only (not
forwarded to dependents). `[build-dependencies]` is reserved for future
build-script support; in v0.5.3 it is parsed but no build-script
invocation is specified.

Each entry is either a version string (`name = "0.3"`) or an inline
table:

| Field | Type | Meaning |
|---|---|---|
| `version` | string | SemVer requirement. Defaults to `"*"` if absent and another source is set. |
| `features` | list | Features to enable on the dep. |
| `default-features` | bool | Disable the dep's `default` feature group. Defaults to `true`. |
| `optional` | bool | Only linked when activated by a `[features]` entry via `"dep:name"` or `"name/feat"`. |
| `git` | string | Git source URL. Mutually exclusive with `path`. |
| `rev` | string | **Required** when `git` is set. Commit SHA only — branches and tags are rejected as non-reproducible. |
| `path` | string | Workspace-internal source. Path must resolve inside the same workspace. |

Source precedence: `path` > `git` > registry. Setting more than one
source on a single entry is a manifest error.

### `[features]`

Additive sets of conditional-compilation flags. Members:

- `"name"` activates the local feature `name`.
- `"crate/feature"` activates `feature` on dependency `crate`.
- `"dep:crate"` activates the optional dependency `crate` without
  enabling any same-named local feature.

`default = [...]` is the implicit feature group, disabled by setting
`default-features = false` on the consuming side. `cfg(feature = "name")`
(§12) is the only in-source way to test feature state.

### `[target.<target>.dependencies]`

One section per build target (`native`, `wasm`, `evm`, `svm`). Same shape
as `[dependencies]`; merged additively. Conflicting `version`/`source`
between base and target sections is a manifest error; only `features`
and `default-features` may differ. `[target.<target>.dev-dependencies]`
and `[target.<target>.build-dependencies]` are accepted with the same
merge rule.

On-chain prohibitions (§11.1, §12.3) are not bypassed by declaring a dep
under `[target.evm.dependencies]`.

### `[targets]`

Project-level default target configuration. Distinct from
`[target.<target>.dependencies]` (different role).

| Field | Type | Default | Meaning |
|---|---|---|---|
| `default` | string | `"native"` | Target used when `--target` is omitted. |
| `contracts` | list | `[]` | On-chain target set (subset of `["evm", "svm"]`). |

### `[profile.<name>]`

Four built-in profiles:

| Profile | Used by | Default `inherits` |
|---|---|---|
| `dev` | `sploosh build` (no `--release`) | — |
| `release` | `sploosh build --release` | — |
| `test` | `sploosh test` | `dev` |
| `bench` | `sploosh test --bench` | `release` |

Knob types and defaults:

| Knob | Type | `dev` | `release` | Notes |
|---|---|---|---|---|
| `opt-level` | `0`–`3`, `"s"`, `"z"` | `0` | `3` | `"s"` and `"z"` size-optimize. |
| `lto` | `false` / `"thin"` / `"fat"` | `false` | `"thin"` | ThinLTO is parallel; `"fat"` is whole-program. |
| `debug` | `0` / `1` / `2` / `false` | `2` | `0` | `1` = line tables; `2` = full. |
| `strip` | `"none"` / `"debuginfo"` / `"symbols"` | `"none"` | `"debuginfo"` | Applied at link. |
| `incremental` | bool | `true` | `false` | Incremental compilation cache. |
| `overflow-checks` | bool | `true` | `true` | **Frozen `true` for `evm`/`svm` builds**, overrides any user setting and emits a warning. |

Custom profiles inherit one built-in (or another custom profile,
transitively):

```toml
[profile.release-small]
inherits = "release"
opt-level = "z"
lto = "fat"
strip = "symbols"
```

`codegen-units` and `panic` are intentionally **not** exposed —
`codegen-units` is an LLVM-specific implementation detail; Sploosh's
failure model (§4.8) is fixed, so there is no abort/unwind choice.

Per-target profile overrides (e.g., `[profile.release.evm]`) are not
permitted in v0.5.3. Use `#[cfg(target = "evm")]` and feature flags for
target-specific code paths.

### `[runtime]`

Native/wasm runtime tunables. Silently ignored for `evm`/`svm` builds.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `threads` | integer ≥ 1 | one per CPU core | M:N scheduler thread count (§8.10). |
| `mailbox_default_capacity` | integer ≥ 1 | `1024` | Default mailbox capacity for actors without `@mailbox(capacity: N)`. |

### `[workspace]`

Workspace-root manifests have **no `[project]` table** — the root is not
itself a buildable package.

| Section | Meaning |
|---|---|
| `[workspace]` | Marker. |
| `members` | Relative paths (globs allowed). |
| `exclude` | Paths under `members` globs to skip. |
| `resolver` | Always `"2"` in v0.5.3. Future resolver versions are an explicit opt-in. |
| `[workspace.package]` | Default `[project]` field values for members. |
| `[workspace.dependencies]` | Pre-resolved version requirements for member inheritance. |

Members consume defaults via `field.workspace = true`. Inherited deps
may add features locally:

```toml
sploosh_chain = { workspace = true, features = ["svm"] }
```

A workspace has exactly one `sploosh.lock` at the root; member
lockfiles are rejected.

## Lockfile (`sploosh.lock`)

TOML, with one `[[package]]` array entry per resolved package:

```toml
version = 1

[[package]]
name = "sploosh_web"
version = "0.3.2"
source = "registry+https://packages.sploosh.dev"
checksum = "blake3:K6Y2QF3RZBXWNYV2T3X6UQEI5JJ4J6S7NTWWF7PADGUZB6E5W2KQ"
dependencies = ["sploosh_proto"]
```

- **Hash algorithm: Blake3.** 32-byte digest, RFC 4648 base32 without
  padding, prefix `"blake3:"`. Hashed over the source archive (registry)
  or the resolved git tree (`git`).
- Entries sorted alphabetically by `name`, then `version`.
- LF line endings; no trailing whitespace.
- Schema `version = 1`; tools must refuse unknown values.
- `sploosh build` / `test` / `check` *verify* the lockfile; mismatch
  fails the build (reserved diagnostic slot `E14xx`). `sploosh update`
  is the only command that may rewrite `sploosh.lock`.

## Dependency sources

| Source | Form | Reproducibility |
|---|---|---|
| Registry | `name = "0.3"` | Resolved version + Blake3 checksum recorded in lockfile. |
| Git | `{ git = "...", rev = "<sha>" }` | `rev` required; branches/tags rejected. |
| Path | `{ path = "../local" }` | Workspace-internal only. |

The default registry endpoint, authentication, and publishing flow are
**deferred to v0.6+**.
