param(
    [string]$Image = "sploosh-dev",
    [switch]$Build
)

$ErrorActionPreference = "Stop"

if ($Build) {
    docker build --pull -t $Image .
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

$workspace = (Get-Location).ProviderPath
$check = "cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace"

$dockerArgs = @(
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

& docker @dockerArgs
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
