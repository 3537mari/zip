$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot\..\.."
Write-Host "=== Offline ZIP install (Windows) ==="
Push-Location "$root\bootstrap"
& .\build.ps1
$dest = Join-Path $env:LOCALAPPDATA "ZIP\bin"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item .\zipc.exe $dest -Force
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$dest*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$dest", "User")
}
Write-Host "Installed zipc to $dest"
Pop-Location
