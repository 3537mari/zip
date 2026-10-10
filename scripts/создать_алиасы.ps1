# Создаёт кириллические hardlink/symlink на ASCII-бинарники после сборки.
$ErrorActionPreference = "Continue"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

cargo build --release --manifest-path (Join-Path $Root "кириллица-алиас/Cargo.toml") | Out-Null

$aliasCandidates = @(
    (Join-Path $Root "кириллица-алиас/target/release/cyrillic-alias.exe"),
    (Join-Path $Root "zipc/target/release/cyrillic-alias.exe"),
    (Join-Path $Root "кириллица-алиас/target/release/cyrillic-alias")
)
$aliasExe = $aliasCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $aliasExe) {
    Write-Host "cyrillic-alias не собран"
    exit 1
}

$dirs = @()
if ($env:CARGO_TARGET_DIR) {
    $dirs += (Join-Path $env:CARGO_TARGET_DIR "release")
}
$dirs += @(
    (Join-Path $Root "zipc/target/release"),
    (Join-Path $Root "зип-профиль/target/release"),
    (Join-Path $Root "зип-отладчик/target/release"),
    (Join-Path $Root "зип-lsp/target/release"),
    (Join-Path $Root "зипм/target/release")
)

$seen = @{}
foreach ($d in $dirs) {
    if (-not (Test-Path $d)) { continue }
    $key = (Resolve-Path $d).Path
    if ($seen.ContainsKey($key)) { continue }
    $seen[$key] = $true
    Write-Host "=== $key ==="
    & $aliasExe $key
}
