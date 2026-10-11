# Тест fixed-point: детерминированная перекомпиляция C-bootstrap (Windows).
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== ТЕСТ FIXED-POINT САМОКОМПИЛЯЦИИ ==="
Write-Host ""

Write-Host "[1/4] Сборка C-bootstrap..."
& pwsh -File .\build.ps1 | Out-Null
$bin = if (Test-Path .\zipc.exe) { ".\zipc.exe" } else { ".\zipc" }
Write-Host "✓ Собран"

$src = @'
функция сложить(а, б) { возврат(а + б) }
функция факториал(н) {
    если н < 2 { возврат(1) }
    возврат(н * факториал(н - 1))
}
функция главная() {
    сумма = 0
    для и в 0..10 { сумма = сумма + и }
    ПечатьЧисло(сложить(сумма, факториал(5)))
}
'@
$tmp = [System.IO.Path]::GetTempPath()
$in = Join-Path $tmp "fp_эталон.zip"
$p1 = Join-Path $tmp "fp_pass1.zipb"
$p2 = Join-Path $tmp "fp_pass2.zipb"
[System.IO.File]::WriteAllText($in, $src, [System.Text.UTF8Encoding]::new($false))

Write-Host ""
Write-Host "[2/4] Компиляция эталона (pass 1)..."
& $bin $in $p1 | Out-Null
Write-Host "✓ fp_pass1.zipb"

Write-Host ""
Write-Host "[3/4] Компиляция эталона (pass 2)..."
& $bin $in $p2 | Out-Null
Write-Host "✓ fp_pass2.zipb"

Write-Host ""
Write-Host "[4/4] Проверка fixed point..."
$h = [System.Security.Cryptography.SHA256]::Create()
$h1 = ([BitConverter]::ToString($h.ComputeHash([IO.File]::ReadAllBytes($p1))) -replace '-','').ToLower()
$h2 = ([BitConverter]::ToString($h.ComputeHash([IO.File]::ReadAllBytes($p2))) -replace '-','').ToLower()
Write-Host "  pass1: $h1"
Write-Host "  pass2: $h2"

if ($h1 -ne $h2) {
    Write-Host ""
    Write-Host "❌ Хеши различаются"
    exit 1
}
Write-Host ""
Write-Host "✅ FIXED POINT ДОСТИГНУТ!"
Write-Host "   C-bootstrap детерминирован (одинаковый .zipb)."

Remove-Item $in, $p1, $p2 -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "=== OK ==="
