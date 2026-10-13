# ZIP на ARM64

## Что нужно знать

После v6.1.0 **компилятор ZIP (`selfc.zipb`) платформонезависим**:
- Лексер, парсер, генератор ZIPB — на **чистом ZIP**
- Fixed-point: v2 = v3 = v4 = `03ec79b9...C6A98C59`
- ZIP-байткод **один и тот же** на x86-64 и ARM64

**Для ARM64 нужно только пересобрать Rust-VM.**

## Сборка VM под ARM64 Linux

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu qemu-user qemu-user-static -y

# linker уже в .cargo/config.toml репозитория:
# [target.aarch64-unknown-linux-gnu]
# linker = "aarch64-linux-gnu-gcc"

cargo build --release \
    --target aarch64-unknown-linux-gnu \
    --manifest-path zip-vm/Cargo.toml

file zip-vm/target/aarch64-unknown-linux-gnu/release/zip-vm
# ELF 64-bit LSB executable, ARM aarch64
```

## Запуск на ARM64 (через QEMU)

```bash
# Сначала получите selfc и .zipb на x86-64 (байткод общий)
./bootstrap/zipc zip-self-zip/главное_чистое.zip /tmp/selfc_v1.zipb
./zip-vm/target/release/zip-vm /tmp/selfc_v1.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_v2.zipb

qemu-aarch64 -L /usr/aarch64-linux-gnu \
    ./zip-vm/target/aarch64-unknown-linux-gnu/release/zip-vm \
    /tmp/привет.zipb
```

## Проверка платформонезависимости

```bash
# Компилировать selfc через ARM64 VM
qemu-aarch64 -L /usr/aarch64-linux-gnu \
    ./zip-vm/target/aarch64-unknown-linux-gnu/release/zip-vm \
    /tmp/selfc_v2.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_arm.zipb

# Хеши совпадают с x86-64
sha256sum /tmp/selfc_v2.zipb /tmp/selfc_arm.zipb
# 03ec79b9...C6A98C59
```

## macOS ARM64 (Apple Silicon)

```bash
rustup target add aarch64-apple-darwin
cargo build --release \
    --target aarch64-apple-darwin \
    --manifest-path zip-vm/Cargo.toml
./zip-vm/target/aarch64-apple-darwin/release/zip-vm программа.zipb
```

## Windows ARM64

```bash
rustup target add aarch64-pc-windows-msvc
cargo build --release \
    --target aarch64-pc-windows-msvc \
    --manifest-path zip-vm/Cargo.toml
.\zip-vm\target\aarch64-pc-windows-msvc\release\zip-vm.exe программа.zipb
```

## Флаг `--target`

```bash
zip-vm --target aarch64-unknown-linux-gnu программа.zipb
```

Информационный флаг: ZIPB не меняется от архитектуры. Нужен для отладки и будущих опций (JIT).

## CI

См. [`.github/workflows/arm64.yml`](../.github/workflows/arm64.yml) — сборка и тесты через QEMU на каждом push в `master`.

## Что дальше

- RISC-V — следующий шаг v7
- WASM — v7.x
- Нативная Windows ARM / Apple Silicon в релизах
