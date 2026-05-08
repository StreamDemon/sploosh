# Package Management

> Adding dependencies, resolving versions, and managing the lockfile.
> Schema is authoritative in `LANGUAGE_SPEC.md` §14.1–§14.4; this page
> describes the workflow and CLI surface.

## Adding Dependencies

Edit `sploosh.toml`:

```toml
[dependencies]
sploosh_web   = "0.3"
sploosh_chain = { version = "0.2", features = ["evm"] }

[dev-dependencies]
sploosh_test_utils = "0.1"
```

Then resolve and lock:

```bash
sploosh update
```

`sploosh update` is the only command that may rewrite `sploosh.lock`.
`sploosh build`, `sploosh test`, and `sploosh check` *verify* the lock
file against the manifest and fail the build on any incompatibility (no
silent re-resolution).

## Importing

```sploosh
use sploosh_web::{Server, Router};
```

Imports resolve against the dependency graph computed from the manifest
plus the lockfile.

## Dependency Sources

| Source | Form | Reproducibility |
|---|---|---|
| Registry | `name = "0.3"` (default) | Resolved version + Blake3 checksum recorded in lockfile. |
| Git | `{ git = "...", rev = "<commit-sha>" }` | `rev` is required and must be a commit SHA. Branches and tags are rejected as non-reproducible. |
| Path | `{ path = "../local" }` | Workspace-internal only — paths escaping the workspace are a manifest error. |

Source precedence: `path` > `git` > registry. Setting more than one
source on a single dependency entry is a manifest error.

## Version Requirement Syntax

Matches Cargo:

| Form | Meaning |
|---|---|
| `"0.3"` / `"^0.3"` | Compatible with `0.3.x` (caret semantics, the default). |
| `"~0.3.4"` | At least `0.3.4`, less than `0.4.0`. |
| `"=1.0.0"` | Exact version. |
| `">=0.3, <0.5"` | Comparator chain. |
| `"0.3.*"` | Wildcard within the named segment. |

The resolver is *resolver v2*: features enabled for a dependency in one
target/dev-dep context do not leak into other contexts. Dev-dependency
features are kept separate from non-test feature graphs.

## Workspaces

Workspaces share a single `sploosh.lock` at the root. Members consume
shared version requirements via the `.workspace = true` form:

```toml
# workspace root
[workspace.dependencies]
sploosh_web = "0.3"
```

```toml
# member crate
[dependencies]
sploosh_web.workspace = true
```

See §14.2 for the full workspace contract.

## Lockfile

`sploosh.lock` is TOML with one `[[package]]` array entry per resolved
package. Hashes are **Blake3** (32-byte digest, base32-no-pad, prefixed
`"blake3:"`). Entries are sorted alphabetically by `name`, then
`version`. Schema `version = 1`.

```toml
version = 1

[[package]]
name = "sploosh_web"
version = "0.3.2"
source = "registry+https://packages.sploosh.dev"
checksum = "blake3:K6Y2QF3RZBXWNYV2T3X6UQEI5JJ4J6S7NTWWF7PADGUZB6E5W2KQ"
dependencies = ["sploosh_proto"]
```

**Check the lockfile in for binaries and workspaces.** Library packages
may omit it; if absent, `sploosh build` resolves on the fly without
writing a lockfile.

## Inspecting the Graph

```bash
sploosh tree              # full dependency tree
sploosh tree --target evm # tree as resolved for a specific target
```

## Deferred to v0.6+

- Public registry endpoint, authentication, and publishing flow.
- Build scripts (`[build-dependencies]` is parsed and reserved, but no
  invocation is specified).
- Per-target profile overrides (e.g., `[profile.release.evm]`).
- `sploosh yank` / registry-side version retraction.
