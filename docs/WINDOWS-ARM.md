# ZIP на Windows ARM64

## Что нужно знать

После v6.1.0 **компилятор ZIP (`selfc.zipb`) платформонезависим**:
- Лексер, парсер, генератор ZIPB — на **чистом ZIP**
- Fixed-point: v2 = v3 = v4 = `03ec79b9...c6a98c59`
- ZIP-байткод **один и тот же** на x86-64, ARM64 Linux/macOS, **Windows ARM64**, RISC-V, WASM

**Для Windows ARM64 нужно только пересобрать Rust-VM.**

## Сборка VM под Windows ARM64

```powershell
rustup target add aarch64-pc-windows-msvc

# Требуется Visual Studio / Build Tools с компонентом:
# "MSVC v143+ — VS C++ ARM64/ARM64EC build tools"
# (на GitHub Actions: windows-11-arm — уже есть)

cargo build --release `
    --target aarch64-pc-windows-msvc `
    --manifest-path zip-vm/Cargo.toml
```

На нативном Windows ARM-устройстве достаточно:

```powershell
cargo build --release --manifest-path zip-vm/Cargo.toml
```

## Проверка бинарника

```powershell
# PE machine via dumpbin (если есть)
dumpbin /headers zip-vm\target\aarch64-pc-windows-msvc\release\zip-vm.exe |
    Select-String "machine"
# Ожидание: ARM64 / AA64

# Или без dumpbin (PowerShell):
$exe = "zip-vm\target\aarch64-pc-windows-msvc\release\zip-vm.exe"
$b = [IO.File]::ReadAllBytes((Resolve-Path $exe))
$pe = [BitConverter]::ToInt32($b, 0x3C)
"0x{0:X4}" -f [BitConverter]::ToUInt16($b, $pe + 4)   # → 0xAA64
```

## Запуск на Windows ARM (нативно)

```powershell
.\zip-vm\target\aarch64-pc-windows-msvc\release\zip-vm.exe `
    C:\Temp\привет.zipb
```

## Проверка платформонезависимости

```powershell
.\zip-vm\target\aarch64-pc-windows-msvc\release\zip-vm.exe `
    C:\Temp\selfc_v2.zipb `
    zip-self-zip\главное_чистое.zip C:\Temp\selfc_win_arm.zipb

$hash = (Get-FileHash C:\Temp\selfc_win_arm.zipb -Algorithm SHA256).Hash
# 03EC79B9...C6A98C59 — как на x86-64 / Linux ARM / RISC-V
```

## CI

См. [`.github/workflows/windows-arm.yml`](../.github/workflows/windows-arm.yml):

| Job | Runner | Что делает |
|-----|--------|------------|
| `build-windows-arm` | `windows-11-arm` | Нативная сборка + запуск + fixed-point |
| `cross-build-windows-arm` | `windows-latest` | Кросс-сборка `aarch64-pc-windows-msvc` |

## Что дальше

- Нативные артефакты в GitHub Releases
- Мобильные биндинги: Android / iOS (v8)
