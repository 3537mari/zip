#Requires -Version 7.0
$ErrorActionPreference = "Stop"
Write-Host "=== Build ZIP MSI ===" -ForegroundColor Cyan
$root = Resolve-Path "$PSScriptRoot\..\.."
Push-Location $root
cargo build --release --manifest-path zipc/Cargo.toml
cargo build --release --manifest-path zip-vm/Cargo.toml
Pop-Location
$payload = Join-Path $PSScriptRoot "payload"
New-Item -ItemType Directory -Force -Path $payload | Out-Null
Copy-Item "$root\zipc\target\release\zipc.exe" "$payload\zipc.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$root\zip-vm\target\release\zip-vm.exe" "$payload\zip-vm.exe" -Force -ErrorAction SilentlyContinue
if (-not (Get-Command candle.exe -ErrorAction SilentlyContinue)) {
    Write-Host "WiX not found (candle.exe). Binaries copied to payload/. Install WiX to build MSI." -ForegroundColor Yellow
    exit 0
}
Push-Location $PSScriptRoot
candle.exe zip.wxs -out zip.wixobj
light.exe zip.wixobj -out ZIP-3.0.0-x64.msi
Pop-Location
Write-Host "Done"
