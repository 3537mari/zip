#Requires -Version 7.0
param([string]$Test = "examples/selfhost_test.zip")
$ErrorActionPreference = "Stop"
# ASCII entrypoint → Cyrillic self-host check
$script = Join-Path $PSScriptRoot "проверить_самокомпиляцию.ps1"
if (-not (Test-Path $script)) {
    # Inline fallback (same logic) if Cyrillic path broken
    $Root = Resolve-Path "$PSScriptRoot\.."
    Set-Location $Root
    $Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
    $Spec = Join-Path $Root "zip-self\native.zip"
    & $Zipc $Spec -o (Join-Path $Root "zip-self\selfc_v1") --llvm | Out-Null
    & (Join-Path $Root "zip-self\selfc_v1.exe") $Spec (Join-Path $Root "zip-self\selfc_v2") | Out-Null
    & (Join-Path $Root "zip-self\selfc_v2.exe") $Spec (Join-Path $Root "zip-self\selfc_v3") | Out-Null
    $h2 = (Get-FileHash "zip-self\selfc_v2.exe" -Algorithm SHA256).Hash
    $h3 = (Get-FileHash "zip-self\selfc_v3.exe" -Algorithm SHA256).Hash
    Write-Host "v2=$h2"
    Write-Host "v3=$h3"
    if ($h2 -ne $h3) { exit 1 }
    & (Join-Path $Root "zip-self\selfc_v3.exe") (Join-Path $Root $Test) (Join-Path $Root "zip-self\out_check") | Out-Null
    exit 0
}
& $script
