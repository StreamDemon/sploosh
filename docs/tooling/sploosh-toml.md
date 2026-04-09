# sploosh.toml

> Project manifest format.

## Full Example

```toml
[project]
name = "my-app"
version = "0.1.0"
edition = "2026"

[dependencies]
sploosh_web = "0.3"
sploosh_db = "0.2"

[features]
default = ["json"]
json = []
postgres = ["sploosh_db"]

[targets]
default = "native"
contracts = ["evm", "svm"]
```

## Sections

### [project]
- `name` -- package name
- `version` -- semver version string
- `edition` -- language edition year

### [dependencies]
Key-value pairs of package name to version requirement.

### [features]
Feature flags for conditional compilation. Used with `#[cfg(feature = "name")]`.

### [targets]
- `default` -- default build target (`native`, `wasm`, `evm`, `svm`)
- `contracts` -- list of on-chain targets

<!-- TODO: Expand with workspace support, dev-dependencies, build profiles once implemented -->
