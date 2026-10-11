# Сравнение C-bootstrap с Rust zipc (Windows)
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== СРАВНЕНИЕ C-BOOTSTRAP С RUST-BOOTSTRAP ==="
Write-Host ""

Write-Host "[1/4] Сборка C-bootstrap..."
& pwsh -File .\build.ps1 | Out-Null
$cBin = ".\zipc.exe"

Write-Host ""
Write-Host "[2/4] Проверка Rust zipc..."
$rust = "..\zipc\target\release\zipc.exe"
if (-not (Test-Path $rust)) {
    Push-Location ..\zipc
    cargo build --release --quiet
    Pop-Location
}

Write-Host ""
Write-Host "[3/4] Компиляция одинакового файла..."
$src = @'
функция сложить(а, б) {
    возврат(а + б)
}

функция главная() {
    ПечатьЧисло(сложить(2, 3))
}
'@
$tmp = [System.IO.Path]::GetTempPath()
$in = Join-Path $tmp "тест_сравнение.zip"
$cOut = Join-Path $tmp "из_c.zipb"
$rBase = Join-Path $tmp "из_rust"
[System.IO.File]::WriteAllText($in, $src, [System.Text.UTF8Encoding]::new($false))

& $cBin $in $cOut | Out-Null
& $rust байткод $in -o $rBase | Out-Null
$rOut = if (Test-Path "$rBase.zipb") { "$rBase.zipb" } else { $rBase }

Write-Host ""
Write-Host "[4/4] Сравнение результатов..."
$szC = (Get-Item $cOut).Length
$szR = (Get-Item $rOut).Length
Write-Host "  C-bootstrap:    $szC байт"
Write-Host "  Rust-bootstrap: $szR байт"
if ($szC -eq $szR) { Write-Host ""; Write-Host "✅ Размеры совпадают!" }
else { Write-Host ""; Write-Host "⚠️  Размеры различаются (разный уровень оптимизации / layout)" }

$vm = "..\zip-vm\target\release\zip-vm.exe"
if (Test-Path $vm) {
    $out = & $vm $cOut 2>$null | Where-Object { $_ -match '^\d+$' } | Select-Object -First 1
    Write-Host "  C → VM вывод: $out (ожидание 5)"
}

Remove-Item $in, $cOut -ErrorAction SilentlyContinue
Remove-Item $rOut -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "=== OK ==="
