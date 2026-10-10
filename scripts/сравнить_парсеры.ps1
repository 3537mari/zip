#Requires -Version 7.0
param([string]$Исходник = "examples/простая.zip")

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
$Src = Join-Path $Root $Исходник
if (-not (Test-Path $Src)) {
    $alt = Join-Path $Root "examples\simple.zip"
    if (Test-Path $alt) { $Src = $alt } else { $Src = $Исходник }
}

Write-Host "=== Rust-парсер ==="
& $Zipc $Src --только-парсинг

Write-Host ""
Write-Host "=== ZIP-парсер ==="
& $Zipc (Join-Path $Root "zip-self\парсер.zip") -o (Join-Path $Root "zip-self\парсер_zip") --llvm | Out-Null
$exe = Join-Path $Root "zip-self\парсер_zip.exe"
if (-not (Test-Path $exe)) { $exe = Join-Path $Root "zip-self\parser_zip.exe" }
& $exe $Src
