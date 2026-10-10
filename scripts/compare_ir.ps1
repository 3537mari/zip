#Requires -Version 7.0
param([string]$Source = "examples/ir_test.zip")

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
$Src = Join-Path $Root $Source
if (-not (Test-Path $Src)) { $Src = $Source }

Write-Host "=== Rust IR ==="
& $Zipc $Src --dump-ir

Write-Host ""
Write-Host "=== ZIP IR ==="
& $Zipc (Join-Path $Root "zip-self\ir.zip") -o (Join-Path $Root "zip-self\ir_zip") --llvm | Out-Host
& (Join-Path $Root "zip-self\ir_zip.exe") $Src
