# ASCII alias for compare script
param([string]$Source = "examples/привет.zip")
$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"

# resolve привет.zip
$Src = Join-Path $Root $Source
if (-not (Test-Path $Src)) {
  $cands = Get-ChildItem (Join-Path $Root "examples") -Filter "*.zip" | Where-Object { $_.Name -like "привет*" }
  if ($cands) { $Src = $cands[0].FullName }
}

Write-Host "=== Rust-lekser ==="
& $Zipc $Src --только-лексика

Write-Host ""
Write-Host "=== ZIP-lekser ==="
& $Zipc (Join-Path $Root "zip-self\lexer.zip") -o (Join-Path $Root "zip-self\lekser_zip") --llvm | Out-Host
$exe = Join-Path $Root "zip-self\lekser_zip.exe"
& $exe $Src
