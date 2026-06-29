param(
    [string]$Image = "sploosh-dev",
    [switch]$Build
)

$ErrorActionPreference = "Stop"

if ($Build) {
    docker build -t $Image .
}

$workspace = (Get-Location).ProviderPath
$check = "cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace"

$args = @(
    "run",
    "--rm",
    "-v", "${workspace}:/workspace",
    "-w", "/workspace",
    "-e", "CARGO_INCREMENTAL=0",
    $Image,
    "bash",
    "-c",
    $check
)

& docker @args
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
