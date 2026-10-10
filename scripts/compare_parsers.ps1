#Requires -Version 7.0
param([string]$Source = "examples/simple.zip")

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
$Src = Join-Path $Root $Source
if (-not (Test-Path $Src)) { $Src = $Source }

Write-Host "=== Rust-parser ==="
& $Zipc $Src --parse-only

Write-Host ""
Write-Host "=== ZIP-parser ==="
& $Zipc (Join-Path $Root "zip-self\parser.zip") -o (Join-Path $Root "zip-self\parser_zip") --llvm | Out-Host
& (Join-Path $Root "zip-self\parser_zip.exe") $Src
