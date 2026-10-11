#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$clang = "C:\Program Files\LLVM\bin\clang.exe"
if (-not (Test-Path $clang)) {
    $c = Get-Command clang -ErrorAction SilentlyContinue
    if ($c) { $clang = $c.Source } else { throw "clang not found" }
}
$src = @("утилиты.c","таблицы.c","лексер.c","парсер.c","генератор.c","дамп.c","проверка.c","главное.c")
Push-Location $PSScriptRoot
& $clang -O3 -Wall -Wextra -std=c11 -D_CRT_SECURE_NO_WARNINGS -o zipc.exe @src
if ($LASTEXITCODE -ne 0) { throw "clang failed" }
$lines = 0
foreach ($f in @("zipb.h") + $src) { $lines += (Get-Content $f).Count }
Write-Host "OK: $PSScriptRoot\zipc.exe ($lines LOC)"
Pop-Location
