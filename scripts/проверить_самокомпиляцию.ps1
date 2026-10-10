#Requires -Version 7.0
$ErrorActionPreference = "Stop"

$Root = Resolve-Path "$PSScriptRoot\.."
Set-Location $Root

Write-Host "=== ПРОВЕРКА ПОЛНОЙ САМОКОМПИЛЯЦИИ ZIP ==="
Write-Host ""

$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
$Spec = Join-Path $Root "zip-self\нативный.zip"
if (-not (Test-Path $Spec)) { $Spec = Join-Path $Root "zip-self\native.zip" }
$Test = Join-Path $Root "examples\selfhost_test.zip"
if (-not (Test-Path $Test)) { $Test = Join-Path $Root "examples\самокомпиляция_тест.zip" }

# Итерация 1
Write-Host "[1/3] Компиляция selfc_v1 через Rust-бутстрэп..."
& $Zipc $Spec -o (Join-Path $Root "zip-self\selfc_v1") --llvm | Out-Null
if (-not (Test-Path "zip-self\selfc_v1.exe")) { throw "selfc_v1 не собран" }
$hash1 = (Get-FileHash "zip-self\selfc_v1.exe" -Algorithm SHA256).Hash
Write-Host "  SHA256: $hash1"

# smoke
& "zip-self\selfc_v1.exe" $Test "zip-self\out_v1" | Out-Null
$msg1 = & "zip-self\out_v1.exe" | Out-String
if ($msg1 -notmatch "Привет") { throw "selfc_v1 smoke failed: $msg1" }

# Итерация 2
Write-Host "[2/3] Компиляция selfc_v2 через selfc_v1..."
& "zip-self\selfc_v1.exe" $Spec "zip-self\selfc_v2" | Out-Null
if (-not (Test-Path "zip-self\selfc_v2.exe")) { throw "selfc_v2 не собран" }
$hash2 = (Get-FileHash "zip-self\selfc_v2.exe" -Algorithm SHA256).Hash
Write-Host "  SHA256: $hash2"

& "zip-self\selfc_v2.exe" $Test "zip-self\out_v2" | Out-Null
$msg2 = & "zip-self\out_v2.exe" | Out-String
if ($msg2 -notmatch "Привет") { throw "selfc_v2 smoke failed: $msg2" }

# Итерация 3
Write-Host "[3/3] Компиляция selfc_v3 через selfc_v2..."
& "zip-self\selfc_v2.exe" $Spec "zip-self\selfc_v3" | Out-Null
if (-not (Test-Path "zip-self\selfc_v3.exe")) { throw "selfc_v3 не собран" }
$hash3 = (Get-FileHash "zip-self\selfc_v3.exe" -Algorithm SHA256).Hash
Write-Host "  SHA256: $hash3"

& "zip-self\selfc_v3.exe" $Test "zip-self\out_v3" | Out-Null
$msg3 = & "zip-self\out_v3.exe" | Out-String
if ($msg3 -notmatch "Привет") { throw "selfc_v3 smoke failed: $msg3" }

Write-Host ""
Write-Host "v1 size: $((Get-Item zip-self\selfc_v1.exe).Length)"
Write-Host "v2 size: $((Get-Item zip-self\selfc_v2.exe).Length)"
Write-Host "v3 size: $((Get-Item zip-self\selfc_v3.exe).Length)"
Write-Host ""

if ($hash2 -eq $hash3) {
    Write-Host "САМОКОМПИЛЯЦИЯ ПОДТВЕРЖДЕНА (selfc_v2 == selfc_v3)" -ForegroundColor Green
    if ($hash1 -eq $hash2) {
        Write-Host "Также v1 == v2 (бутстрэп совпал с фиксированной точкой)" -ForegroundColor Green
    }
    exit 0
} else {
    Write-Host "Хеши selfc_v2 и selfc_v3 различаются" -ForegroundColor Yellow
    Write-Host "  Это может быть из-за встроенных timestamp в PE-заголовке"
    Write-Host "  Проверь байт-в-байт / флаг -Wl,/Brepro"
    exit 1
}
