# PowerShell с UTF-8 (рекомендуется 7+; работает и на 5.1)
$ErrorActionPreference = "Stop"

# Установка UTF-8 для всех операций
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
$PSDefaultParameterValues['*:Encoding'] = 'utf8'
try {
    chcp 65001 | Out-Null
} catch {}

$env:RUSTFLAGS = "-C target-cpu=native"

Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== Сборка ZIP (UTF-8) ==="

# Сборка компилятора
cargo build --release --manifest-path zipc/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Сборка VM
cargo build --release --manifest-path zip-vm/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Компилятор ZIP: zipc/target/release/zipc.exe"
Write-Host "VM ZIP:         zip-vm/target/release/zip-vm.exe"
Write-Host ""
Write-Host "Подсказка: для кириллических флагов используйте ASCII-алиасы:"
Write-Host "  zipc файл.zip --bytecode -o выход"
Write-Host "  zipc файл.zip --llvm -o выход"
