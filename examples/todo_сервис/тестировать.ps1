#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Here = $PSScriptRoot
$Zipc = Join-Path $Root "zipc\target\release\zipc.exe"
if ($env:CARGO_TARGET_DIR) {
    $c = Join-Path $env:CARGO_TARGET_DIR "release\zipc.exe"
    if (Test-Path $c) { $Zipc = $c }
}
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
            [System.Environment]::GetEnvironmentVariable("Path", "User")

Write-Host "=== ТЕСТИРОВАНИЕ TODO-СЕРВИСА ==="

# 1. Unit: хранилище
Write-Host "`n[1] Тест хранилища"
Push-Location $Here
& $Zipc запустить (Join-Path $Here "tests\тест_хранилища.zip")
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "storage test failed" }
Pop-Location

# 2. Сборка сервера
Write-Host "`n[2] Сборка сервера"
$Server = Join-Path $Here "сервер.exe"
& $Zipc собрать (Join-Path $Here "src\главная.zip") -o $Server
if (-not (Test-Path $Server)) { throw "server binary missing" }

# 3. Запуск
Write-Host "`n[3] Запуск сервера на :8080"
$env:ZIP_TODO_DB = Join-Path $Here "данные\задачи.json"
# чистая БД для теста
@'
{
  "следующий_id": 1,
  "задачи": []
}
'@ | Set-Content -Encoding UTF8 (Join-Path $Here "данные\задачи.json")

$proc = Start-Process -FilePath $Server -WorkingDirectory $Here -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 1
if ($proc.HasExited) { throw "server exited early" }

try {
    function Invoke-Json($Method, $Url, $Body = $null, $Token = $null) {
        $headers = @{}
        if ($Token) { $headers["Authorization"] = "Bearer $Token" }
        if ($Body) {
            return Invoke-RestMethod -Method $Method -Uri $Url -Headers $headers -ContentType "application/json; charset=utf-8" -Body $Body
        }
        return Invoke-RestMethod -Method $Method -Uri $Url -Headers $headers
    }

    Write-Host "`n[4] Health"
    $h = Invoke-RestMethod http://localhost:8080/health
    if ($h.статус -ne "ok") { throw "health failed: $($h | ConvertTo-Json)" }
    Write-Host ($h | ConvertTo-Json -Compress)

    Write-Host "`n[5] Вход (ASCII /api/login + кириллические поля JSON)"
    $login = Invoke-Json POST http://localhost:8080/api/login '{"имя":"admin","пароль":"admin"}'
    $tok = $login.токен
    if (-not $tok) { throw "no token" }
    Write-Host "Токен: $($tok.Substring(0, [Math]::Min(16, $tok.Length)))..."

    Write-Host "`n[5b] Вход по /api/вход (URL-encoded)"
    $login2 = Invoke-Json POST ("http://localhost:8080/api/" + [uri]::EscapeDataString("вход")) '{"имя":"admin","пароль":"admin"}'
    if (-not $login2.токен) { throw "cyrillic login path failed" }

    Write-Host "`n[6] Создание задачи"
    $created = Invoke-Json POST http://localhost:8080/api/tasks '{"заголовок":"Купить молоко","описание":"2 литра"}' $tok
    Write-Host ($created | ConvertTo-Json -Compress -Depth 5)
    $id = $created.id

    Write-Host "`n[7] Список"
    $list = Invoke-Json GET http://localhost:8080/api/tasks $null $tok
    $count = if ($list -is [System.Array]) { $list.Count } else { 1 }
    Write-Host "задач: $count"

    Write-Host "`n[8] GET по id"
    $one = Invoke-Json GET "http://localhost:8080/api/tasks/$id" $null $tok
    Write-Host ($one | ConvertTo-Json -Compress -Depth 5)

    Write-Host "`n[9] PUT выполнена"
    $upd = Invoke-Json PUT "http://localhost:8080/api/tasks/$id" '{"выполнена":true}' $tok
    if (-not $upd.выполнена) { throw "update failed" }
    Write-Host ($upd | ConvertTo-Json -Compress -Depth 5)

    Write-Host "`n[10] DELETE"
    $del = Invoke-Json DELETE "http://localhost:8080/api/tasks/$id" $null $tok
    Write-Host ($del | ConvertTo-Json -Compress -Depth 5)

    Write-Host "`n[11] 401 без токена"
    try {
        Invoke-WebRequest http://localhost:8080/api/tasks -UseBasicParsing | Out-Null
        throw "expected 401"
    } catch {
        Write-Host "auth check OK ($($_.Exception.Message.Split("`n")[0]))"
    }

    Write-Host "`n=== ГОТОВО: все сценарии OK ==="
} finally {
    if ($proc -and -not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force
    }
}
