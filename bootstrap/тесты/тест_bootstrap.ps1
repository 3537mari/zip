$ErrorActionPreference = "Stop"
$boot = Split-Path $PSScriptRoot -Parent
Set-Location $boot
& "$boot\build.ps1"
$bin = Join-Path $boot "zipc.exe"
$in = Join-Path $boot "t_hello.zip"
$out = Join-Path $boot "t_hello.zipb"
Set-Content -Path $in -Value "ПечатьЧисло(42)" -Encoding utf8NoBOM
& $bin $in $out
if (-not (Test-Path $out)) { throw "missing $out" }
$bytes = [IO.File]::ReadAllBytes($out)
$magic = [Text.Encoding]::ASCII.GetString($bytes[0..3])
if ($magic -ne "ZIPB") { throw "bad magic: $magic" }
$ver = [BitConverter]::ToUInt16($bytes, 4)
if ($ver -ne 3) { throw "bad version: $ver" }
Write-Host "bootstrap test OK ($($bytes.Length) bytes, magic=$magic ver=$ver)"
Remove-Item $in, $out -ErrorAction SilentlyContinue
