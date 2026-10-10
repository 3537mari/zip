# Сравнивает вывод Rust-лексера и ZIP-лексера
param([string]$Исходник = "$PSScriptRoot\..\examples\vm_demo.zip")
$Root = Resolve-Path "$PSScriptRoot\.."
$Zipc = if (Test-Path "$Root\zipc\target\release\zipc.exe") {
  "$Root\zipc\target\release\zipc.exe"
} else { "$Root\zipc\target\debug\zipc.exe" }

Write-Host "=== Rust-lekser ==="
& $Zipc $Исходник --только-лексика

Write-Host ""
Write-Host "=== ZIP-lekser ==="
& $Zipc "$Root\zip-self\лексика.zip" -o "$Root\zip-self\лексер_zip" --llvm
& "$Root\zip-self\лексер_zip.exe" $Исходник
