param([switch]$SkipChecks)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)

function Invoke-Cargo {
    & cargo @args
    if ($LASTEXITCODE -ne 0) { throw "cargo failed with exit code $LASTEXITCODE" }
}

if (-not $SkipChecks) {
    Invoke-Cargo fmt --all -- --check
    Invoke-Cargo clippy --workspace --all-targets --locked -- -D warnings
    Invoke-Cargo test --workspace --locked
}
Invoke-Cargo tauri build --target x86_64-pc-windows-msvc --bundles nsis -- --locked
$installer = Get-ChildItem 'target/x86_64-pc-windows-msvc/release/bundle/nsis/*-setup.exe'
if ($installer.Count -ne 1) { throw 'Expected exactly one NSIS installer.' }
New-Item -ItemType Directory -Force outputs | Out-Null
Copy-Item -LiteralPath $installer.FullName -Destination outputs
Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256
