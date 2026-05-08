# Runbook: New Project Setup

> Step-by-step guide to creating a new Sploosh project from scratch.

## Steps

1. **Create the project:**
   ```bash
   sploosh new my-project
   cd my-project
   ```

2. **Review the generated structure:**
   ```
   my-project/
   ├── sploosh.toml
   └── src/
       └── main.sp
   ```

3. **Edit `sploosh.toml`** to add dependencies and configure targets.

4. **Write your entry point** in `src/main.sp`:
   ```sploosh
   fn main() -> Result<(), AppError> {
       print("Hello, Sploosh!");
       Ok(())
   }
   ```

5. **Build and run:**
   ```bash
   sploosh run
   ```

6. **Initialize git:**
   ```bash
   git init
   git add .
   git commit -m "Initial project setup"
   ```

## Generated `sploosh.toml`

```toml
[project]
name = "my-project"
version = "0.1.0"
edition = "0.5"

[dependencies]

[targets]
default = "native"
```

Add `[dev-dependencies]`, target-specific deps under
`[target.<target>.dependencies]`, or build profiles under `[profile.<name>]`
as the project grows. See `docs/tooling/sploosh-toml.md` for the full
schema and §14.1 of `LANGUAGE_SPEC.md` for the authoritative contract.

## Workspace Bootstrap

For a multi-crate project, scaffold a workspace root by hand:

```bash
mkdir my-workspace && cd my-workspace
mkdir -p crates contracts
```

Create the root `sploosh.toml` (no `[project]` — the root is not a
buildable package):

```toml
[workspace]
members = ["crates/*", "contracts/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "0.5"
license = "Apache-2.0"

[workspace.dependencies]
sploosh_web   = "0.3"
sploosh_chain = "0.2"
```

Then add member packages:

```bash
cd crates && sploosh new api && cd ..
```

Edit `crates/api/sploosh.toml` to inherit from the workspace:

```toml
[project]
name = "api"
version.workspace = true
edition.workspace = true

[dependencies]
sploosh_web.workspace = true
```

A workspace has exactly one `sploosh.lock` at the root.

<!-- TODO: Expand with common project templates (web API, CLI tool, smart contract) once available -->
