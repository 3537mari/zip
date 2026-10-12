# Финальный тест: fixed-point stage-1+ без C (VM-генератор)
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "../..")
Set-Location $Root

Write-Host "=== ФИНАЛЬНЫЙ ТЕСТ FIXED-POINT БЕЗ C ==="

$Tmp = $env:TEMP
$V1 = Join-Path $Tmp "selfc_v1.zipb"
$V2 = Join-Path $Tmp "selfc_v2.zipb"
$V3 = Join-Path $Tmp "selfc_v3.zipb"
$Src = "zip-self-zip/главное_чистое.zip"

Write-Host "[1/5] C-bootstrap..."
$Zipc = Join-Path $Root "bootstrap/zipc.exe"
if (-not (Test-Path $Zipc)) {
    & pwsh -File (Join-Path $Root "bootstrap/build.ps1") | Out-Null
}
if (-not (Test-Path $Zipc)) { throw "bootstrap/zipc.exe не найден" }
Write-Host "OK $Zipc"

Write-Host "[2/5] C → selfc_v1..."
& $Zipc $Src $V1
if ($LASTEXITCODE -ne 0) { throw "C-bootstrap failed" }

Write-Host "[3/5] selfc_v1 → selfc_v2 (VM, без C)..."
cargo build --release --manifest-path zip-vm/Cargo.toml | Out-Null
$VmCandidates = @(
    (Join-Path $env:CARGO_TARGET_DIR "release/zip-vm.exe"),
    (Join-Path $Root "zip-vm/target/release/zip-vm.exe")
) | Where-Object { $_ -and (Test-Path $_) }
$Vm = $VmCandidates | Sort-Object { (Get-Item $_).LastWriteTime } -Descending | Select-Object -First 1
if (-not $Vm) { throw "zip-vm.exe не найден" }
Write-Host "VM: $Vm"
Remove-Item Env:ZIP_USE_C_BOOTSTRAP -ErrorAction SilentlyContinue
& $Vm $V1 $Src $V2
if ($LASTEXITCODE -ne 0) { throw "v1 compile failed" }

Write-Host "[4/5] selfc_v2 → selfc_v3..."
& $Vm $V2 $Src $V3
if ($LASTEXITCODE -ne 0) { throw "v2 compile failed" }

function Get-Sha256([string]$path) {
    (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLowerInvariant()
}

$H1 = Get-Sha256 $V1
$H2 = Get-Sha256 $V2
$H3 = Get-Sha256 $V3
Write-Host "  v1: $H1"
Write-Host "  v2: $H2"
Write-Host "  v3: $H3"

if ($H2 -eq $H3) {
    Write-Host "FIXED-POINT БЕЗ C (v2 == v3)"
    if ($H1 -ne $H2) {
        Write-Host "v1!=v2 ожидаемо (C stage-0 vs VM gen)"
    }
    exit 0
}
Write-Host "Хеши v2/v3 различаются"
exit 1
