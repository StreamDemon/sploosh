# Compiler Development

Use this when changing compiler crates under `crates/`.

Ubuntu Docker exists as a local CI-parity path. It runs the same Rust checks in
a Linux environment like GitHub Actions, and it avoids Windows-specific Cargo
lock/cache behavior during repeated test runs.

## Preconditions

- Rust 1.91 or newer is installed.
- Optional: Docker is installed for Ubuntu-parity local checks.
- You are on a non-`main` branch.
- `docs/spec-plans/LANGUAGE_SPEC.md` is open to the affected section.

## Steps

1. Confirm the branch and workspace state.

```pwsh
git status --short --branch
```

2. Run the compiler checks.

```pwsh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

3. Run the Ubuntu Docker check before opening a PR when working from Windows.

```pwsh
.\scripts\docker-check.ps1 -Build
```

Use the same image without rebuilding after the first run:

```pwsh
.\scripts\docker-check.ps1
```

The script runs `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, and `cargo test --workspace` inside the
`sploosh-dev` Ubuntu container.

4. Add or update `.sp` corpus fixtures when parser behavior changes.

```pwsh
rg -n "<syntax-shape>" docs/spec-plans/LANGUAGE_SPEC.md tests/corpus
```

5. If compiler behavior disagrees with the spec, fix the compiler first.

If the spec is wrong, land a spec amendment before changing the compiler to match
the new behavior. Sync the required mirror docs in the same PR, including
`LANGUAGE_SPEC_PROMPT_CORE.md`, `LANGUAGE_SPEC_PROMPT_WEB3.md`,
`docs/reference/`, `docs/stdlib/`, and any affected guides or runbooks.

## If Something Goes Wrong

| Symptom | Fix |
|---|---|
| Parser accepts syntax that §16 rejects | Tighten the parser or open a spec amendment first. |
| Parser rejects a spec example | Add the example to `tests/corpus/`, then fix the parser. |
| CI skips Rust checks | Confirm changed paths match `.github/workflows/rust.yml`. |
| Docker check cannot connect to Docker | Start Docker Desktop, then rerun `.\scripts\docker-check.ps1`. |
