# Optional workaround for hosts where Tauri's default 32-bit NSIS compiler
# cannot start (Windows error 623). Run after the first packaging attempt.
# Dot-source this script so NSISDIR remains set for the following build.
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
$archive = Join-Path $repoRoot 'work/tools/nsis-x64.tar.gz'
$toolsDir = Join-Path $repoRoot 'work/tools/nsis-x64'
$compilerCache = Join-Path $repoRoot 'target/.tauri/NSIS/makensis.exe'
if (-not (Test-Path -LiteralPath $compilerCache)) {
    throw 'Run cargo tauri build once first, so the local NSIS cache exists.'
}
New-Item -ItemType Directory -Force (Split-Path $archive), $toolsDir | Out-Null
Invoke-WebRequest 'https://github.com/electron-userland/electron-builder-binaries/releases/download/nsis%402.0.4/nsis-bundle-3.13.tar.gz' -OutFile $archive
$expected = 'c18e9ad9684cf4c5bbd7b768cbf1b056d40771d7bd9ae1b513346452decf89d2'
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLower() -ne $expected) {
    throw 'NSIS archive SHA256 mismatch.'
}
& tar -xzf $archive -C $toolsDir
if ($LASTEXITCODE -ne 0) { throw 'Could not extract NSIS.' }
$env:NSISDIR = Join-Path $toolsDir 'nsis-bundle/windows'
Copy-Item -LiteralPath (Join-Path $env:NSISDIR 'makensis.exe') -Destination $compilerCache -Force
# This compiler locates NSISDIR one directory above its executable when the
# bundler clears environment overrides. Supply its matching headers/stubs there.
foreach ($name in @('Stubs', 'Include', 'Contrib', 'Plugins', 'nsisconf.nsh')) {
    Copy-Item -LiteralPath (Join-Path $env:NSISDIR $name) -Destination (Join-Path $repoRoot 'target/.tauri') -Recurse -Force
}
Write-Output 'Workspace NSIS compiler replaced with the pinned x64 build. Run ./scripts/build.ps1 in this PowerShell session.'
