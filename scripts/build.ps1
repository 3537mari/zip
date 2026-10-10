# ASCII alias: works on Windows PowerShell 5.1+
$ErrorActionPreference = "Stop"

try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    [Console]::InputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8
    chcp 65001 | Out-Null
} catch {}

$env:RUSTFLAGS = "-C target-cpu=native"
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== ZIP build ==="
cargo build --release --manifest-path zipc/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo build --release --manifest-path zip-vm/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "zipc: zipc/target/release/zipc.exe"
Write-Host "zip-vm: zip-vm/target/release/zip-vm.exe"
