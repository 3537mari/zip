#Requires -Version 7.0
param([string]$Исходник = "examples/пп_тест.zip")

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
$Src = Join-Path $Root $Исходник
if (-not (Test-Path $Src)) {
    $alt = Join-Path $Root "examples\ir_test.zip"
    if (Test-Path $alt) { $Src = $alt } else { $Src = $Исходник }
}

Write-Host "=== Rust IR ==="
& $Zipc $Src --вывод-пп

Write-Host ""
Write-Host "=== ZIP IR ==="
& $Zipc (Join-Path $Root "zip-self\промежуток.zip") -o (Join-Path $Root "zip-self\пп_zip") --llvm | Out-Null
$exe = Join-Path $Root "zip-self\пп_zip.exe"
if (-not (Test-Path $exe)) { $exe = Join-Path $Root "zip-self\ir_zip.exe" }
& $exe $Src
