# Fixed-point самокомпиляции zip-self-zip (Windows)
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "../..")
Set-Location $Root

Write-Host "=== ФИНАЛЬНЫЙ ТЕСТ FIXED-POINT (zip-self-zip) ==="

$Src = "zip-self-zip/главное.zip"
$V1 = Join-Path $env:TEMP "selfc_v1.zipb"
$V2 = Join-Path $env:TEMP "selfc_v2.zipb"
$V3 = Join-Path $env:TEMP "selfc_v3.zipb"

$Zipc = if (Test-Path "bootstrap/zipc.exe") { "bootstrap/zipc.exe" }
        elseif (Test-Path "bootstrap/zipc") { "bootstrap/zipc" }
        else {
          & pwsh -File bootstrap/build.ps1 | Out-Null
          "bootstrap/zipc.exe"
        }

Write-Host "[1/4] C-bootstrap → selfc_v1..."
& $Zipc $Src $V1
if ($LASTEXITCODE -ne 0) { throw "zipc failed" }

$Vm = "zip-vm/target/release/zip-vm.exe"
if (-not (Test-Path $Vm)) {
  cargo build --release --manifest-path zip-vm/Cargo.toml
}
if (-not (Test-Path $Vm)) { $Vm = "zip-vm/target/release/zip-vm" }

Write-Host "[2/4] selfc_v1 → selfc_v2..."
& $Vm $V1 $Src $V2
if ($LASTEXITCODE -ne 0) { throw "vm stage1 failed" }

Write-Host "[3/4] selfc_v2 → selfc_v3..."
& $Vm $V2 $Src $V3
if ($LASTEXITCODE -ne 0) { throw "vm stage2 failed" }

function Get-Sha256([string]$Path) {
  (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLowerInvariant()
}
$H1 = Get-Sha256 $V1
$H2 = Get-Sha256 $V2
$H3 = Get-Sha256 $V3
Write-Host "  v1: $H1"
Write-Host "  v2: $H2"
Write-Host "  v3: $H3"

if ($H1 -eq $H2 -and $H2 -eq $H3) {
  Write-Host "FIXED-POINT OK"
  exit 0
}
Write-Host "HASH MISMATCH"
exit 1
