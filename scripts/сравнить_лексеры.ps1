#Requires -Version 7.0
param([string]$Исходник = "examples/привет.zip")

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
$Src = Join-Path $Root $Исходник
if (-not (Test-Path $Src)) { $Src = $Исходник }

Write-Host "=== Rust-лексер ==="
& $Zipc $Src --только-лексика

Write-Host ""
Write-Host "=== ZIP-лексер ==="
& $Zipc (Join-Path $Root "zip-self\лексика.zip") -o (Join-Path $Root "zip-self\лексер_zip") --llvm | Out-Null
$exe = Join-Path $Root "zip-self\лексер_zip.exe"
& $exe $Src
