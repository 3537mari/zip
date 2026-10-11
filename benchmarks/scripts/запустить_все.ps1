#Requires -Version 5.1
# Запуск бенчмарков ZIP / Python / Rust / Go
$ErrorActionPreference = "Continue"  # не рвать весь прогон на одной ошибке
$PSNativeCommandUseErrorActionPreference = $false
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Bench = Join-Path $Root "benchmarks"
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
if ($env:CARGO_TARGET_DIR) {
    $cand = Join-Path $env:CARGO_TARGET_DIR "release\zipc.exe"
    if (Test-Path $cand) { $Zipc = $cand }
}
$OutDir = Join-Path $Bench "_out"
$ResultsJson = Join-Path $Bench "results.json"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# PATH: Go после winget
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
            [System.Environment]::GetEnvironmentVariable("Path", "User")

if (-not (Test-Path $Zipc)) {
    Write-Host "Building zipc --release..."
    cargo build --release --manifest-path (Join-Path $Root "zipc\Cargo.toml")
    if ($env:CARGO_TARGET_DIR) {
        $cand = Join-Path $env:CARGO_TARGET_DIR "release\zipc.exe"
        if (Test-Path $cand) { $Zipc = $cand }
    }
}
Write-Host "zipc: $Zipc"

function Get-LastMs([string]$text) {
    $lines = @($text -split "`r?`n" | Where-Object { $_.Trim() -ne "" })
    foreach ($line in $lines) {
        if ($line -match "train_ms:\s*(\d+)") { return [double]$Matches[1] }
        if ($line -match "ZIP_GEMM_sec:\s*(\d+)") { return [double]$Matches[1] }
    }
    # Последние «короткие» числа (< 1e6) — типичный ms; пропускаем fib-результаты и хеши
    $candidates = @()
    foreach ($line in $lines) {
        if ($line -match "^(\d+(\.\d+)?)$") {
            $cand = [double]$Matches[1]
            if ($cand -lt 1000000) { $candidates += $cand }
        }
    }
    if ($candidates.Count -ge 1) { return $candidates[-1] }
    return $null
}

function Measure-Cmd {
    param(
        [string]$Name,
        [scriptblock]$Build,
        [scriptblock]$Run,
        [string]$ExePath = $null
    )
    Write-Host ">> $Name"
    try {
        & $Build
        if ($ExePath -and -not (Test-Path $ExePath) -and $Name -match "ZIP|Rust|Go") {
            throw "binary missing: $ExePath"
        }
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $output = & $Run 2>&1 | Out-String
        $sw.Stop()
        $wall = $sw.Elapsed.TotalMilliseconds
        $internal = Get-LastMs $output
        $ms = if ($null -ne $internal) { $internal } else { [math]::Round($wall, 2) }
        $size = 0
        if ($ExePath -and (Test-Path $ExePath)) {
            $size = (Get-Item $ExePath).Length
        }
        Write-Host ("   {0:N2} ms (wall {1:N2}) size={2}" -f $ms, $wall, $size)
        return @{ ms = [math]::Round([double]$ms, 2); size = $size; wall = [math]::Round($wall, 2); output = $output }
    } catch {
        Write-Host "   FAIL: $_"
        return @{ ms = $null; size = 0; wall = $null; output = "$_" }
    }
}

$results = @{}

# ---------- Fibonacci ----------
$fib = Join-Path $Bench "fibonacci"
$results["fibonacci"] = @{}

$exe = Join-Path $OutDir "fib_zip.exe"
$results["fibonacci"]["ZIP"] = Measure-Cmd "fibonacci/ZIP" {
    & $Zipc собрать (Join-Path $fib "fibonacci.zip") -o $exe
} {
    & $exe
} $exe

$results["fibonacci"]["Python"] = Measure-Cmd "fibonacci/Python" { } {
    python (Join-Path $fib "fibonacci.py")
}

$exe = Join-Path $OutDir "fib_rs.exe"
$results["fibonacci"]["Rust"] = Measure-Cmd "fibonacci/Rust" {
    rustc -O (Join-Path $fib "fibonacci.rs") -o $exe
} {
    & $exe
} $exe

$exe = Join-Path $OutDir "fib_go.exe"
$results["fibonacci"]["Go"] = Measure-Cmd "fibonacci/Go" {
    go build -o $exe (Join-Path $fib "fibonacci.go")
} {
    & $exe
} $exe

# ---------- GEMM ----------
$gemm = Join-Path $Bench "gemm"
$results["gemm"] = @{}

$exe = Join-Path $OutDir "gemm_zip.exe"
$results["gemm"]["ZIP"] = Measure-Cmd "gemm/ZIP" {
    & $Zipc собрать (Join-Path $gemm "gemm.zip") -o $exe
} {
    & $exe
} $exe

$results["gemm"]["Python"] = Measure-Cmd "gemm/Python" { } {
    python (Join-Path $gemm "gemm.py")
}

$exe = Join-Path $OutDir "gemm_rs.exe"
$results["gemm"]["Rust"] = Measure-Cmd "gemm/Rust" {
    rustc -O (Join-Path $gemm "gemm.rs") -o $exe
} {
    & $exe
} $exe

$exe = Join-Path $OutDir "gemm_go.exe"
$results["gemm"]["Go"] = Measure-Cmd "gemm/Go" {
    go build -o $exe (Join-Path $gemm "gemm.go")
} {
    & $exe
} $exe

# ---------- JSON ----------
$json = Join-Path $Bench "json"
$results["json"] = @{}

$exe = Join-Path $OutDir "json_zip.exe"
$results["json"]["ZIP"] = Measure-Cmd "json/ZIP" {
    & $Zipc собрать (Join-Path $json "json.zip") -o $exe
} {
    & $exe
} $exe

$results["json"]["Python"] = Measure-Cmd "json/Python" { } {
    python (Join-Path $json "json.py")
}

$exe = Join-Path $OutDir "json_rs.exe"
$results["json"]["Rust"] = Measure-Cmd "json/Rust" {
    Push-Location $json
    cargo build --release --quiet
    $built = $null
    if ($env:CARGO_TARGET_DIR) {
        $c1 = Join-Path $env:CARGO_TARGET_DIR "release\json_bench.exe"
        if (Test-Path $c1) { $built = $c1 }
    }
    if (-not $built) {
        $c2 = Join-Path (Get-Location) "target\release\json_bench.exe"
        if (Test-Path $c2) { $built = $c2 }
    }
    if (-not $built) { throw "json_bench.exe not found after cargo build" }
    Copy-Item $built $exe -Force
    Pop-Location
} {
    & $exe
} $exe

$exe = Join-Path $OutDir "json_go.exe"
$results["json"]["Go"] = Measure-Cmd "json/Go" {
    go build -o $exe (Join-Path $json "json.go")
} {
    & $exe
} $exe

# ---------- Strings ----------
$strings = Join-Path $Bench "strings"
$results["strings"] = @{}

$exe = Join-Path $OutDir "strings_zip.exe"
$results["strings"]["ZIP"] = Measure-Cmd "strings/ZIP" {
    & $Zipc собрать (Join-Path $strings "strings.zip") -o $exe
} {
    & $exe
} $exe

$results["strings"]["Python"] = Measure-Cmd "strings/Python" { } {
    python (Join-Path $strings "strings.py")
}

$exe = Join-Path $OutDir "strings_rs.exe"
$results["strings"]["Rust"] = Measure-Cmd "strings/Rust" {
    rustc -O (Join-Path $strings "strings.rs") -o $exe
} {
    & $exe
} $exe

$exe = Join-Path $OutDir "strings_go.exe"
$results["strings"]["Go"] = Measure-Cmd "strings/Go" {
    go build -o $exe (Join-Path $strings "strings.go")
} {
    & $exe
} $exe

# ---------- HTTP ----------
$http = Join-Path $Bench "http"
$results["http"] = @{}

$exe = Join-Path $OutDir "http_zip.exe"
$results["http"]["ZIP"] = Measure-Cmd "http/ZIP" {
    & $Zipc собрать (Join-Path $http "http.zip") -o $exe
} {
    & $exe
} $exe

$results["http"]["Python"] = Measure-Cmd "http/Python" { } {
    python (Join-Path $http "http.py")
}

$exe = Join-Path $OutDir "http_rs.exe"
$results["http"]["Rust"] = Measure-Cmd "http/Rust" {
    rustc -O (Join-Path $http "http.rs") -o $exe
} {
    & $exe
} $exe

$exe = Join-Path $OutDir "http_go.exe"
$results["http"]["Go"] = Measure-Cmd "http/Go" {
    go build -o $exe (Join-Path $http "http.go")
} {
    & $exe
} $exe

# ---------- NN ----------
$nn = Join-Path $Bench "nn"
$results["nn"] = @{}

$exe = Join-Path $OutDir "nn_zip.exe"
$results["nn"]["ZIP"] = Measure-Cmd "nn/ZIP" {
    & $Zipc собрать (Join-Path $nn "nn.zip") -o $exe
} {
    & $exe
} $exe

$results["nn"]["Python"] = Measure-Cmd "nn/Python" { } {
    python (Join-Path $nn "nn.py")
}

$exe = Join-Path $OutDir "nn_rs.exe"
$results["nn"]["Rust"] = Measure-Cmd "nn/Rust" {
    rustc -O (Join-Path $nn "nn.rs") -o $exe
} {
    & $exe
} $exe

$exe = Join-Path $OutDir "nn_go.exe"
$results["nn"]["Go"] = Measure-Cmd "nn/Go" {
    go build -o $exe (Join-Path $nn "nn.go")
} {
    & $exe
} $exe

# Save JSON (strip bulky output)
$slim = @{}
foreach ($task in $results.Keys) {
    $slim[$task] = @{}
    foreach ($lang in $results[$task].Keys) {
        $r = $results[$task][$lang]
        $slim[$task][$lang] = @{ ms = $r.ms; size = $r.size; wall = $r.wall }
    }
}
($slim | ConvertTo-Json -Depth 6) | Set-Content -Path $ResultsJson -Encoding UTF8
Write-Host "Saved $ResultsJson"

python (Join-Path $PSScriptRoot "сравнить.py")
python (Join-Path $PSScriptRoot "графики.py")
python (Join-Path $PSScriptRoot "сгенерировать_отчёт.py")
Write-Host "DONE"
