# ZIP на RISC-V

## Что нужно знать

После v6.1.0 **компилятор ZIP (`selfc.zipb`) платформонезависим**:
- Лексер, парсер, генератор ZIPB — на **чистом ZIP**
- Fixed-point: v2 = v3 = v4 = `03ec79b9...c6a98c59`
- ZIP-байткод **один и тот же** на x86-64, ARM64 и RISC-V

**Для RISC-V нужно только пересобрать Rust-VM.**

## Сборка VM под RISC-V Linux

```bash
rustup target add riscv64gc-unknown-linux-gnu
sudo apt install gcc-riscv64-linux-gnu qemu-user qemu-user-static -y

# linker уже в .cargo/config.toml репозитория:
# [target.riscv64gc-unknown-linux-gnu]
# linker = "riscv64-linux-gnu-gcc"

cargo build --release \
    --target riscv64gc-unknown-linux-gnu \
    --manifest-path zip-vm/Cargo.toml

file zip-vm/target/riscv64gc-unknown-linux-gnu/release/zip-vm
# ELF 64-bit LSB executable, UCB RISC-V
```

## Запуск на RISC-V (через QEMU)

```bash
qemu-riscv64 -L /usr/riscv64-linux-gnu \
    ./zip-vm/target/riscv64gc-unknown-linux-gnu/release/zip-vm \
    /tmp/привет.zipb
```

## Проверка платформонезависимости

```bash
qemu-riscv64 -L /usr/riscv64-linux-gnu \
    ./zip-vm/target/riscv64gc-unknown-linux-gnu/release/zip-vm \
    /tmp/selfc_v2.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_riscv.zipb

sha256sum /tmp/selfc_v2.zipb /tmp/selfc_riscv.zipb
# 03ec79b9...c6a98c59 (обе платформы)
```

## Флаг `--target`

```bash
zip-vm --target riscv64gc-unknown-linux-gnu программа.zipb
```

Информационный флаг: ZIPB не меняется от архитектуры.

## CI

См. [`.github/workflows/riscv.yml`](../.github/workflows/riscv.yml).

## Что дальше

- WASM — см. планы в [ROADMAP_V7.md](../ROADMAP_V7.md)
- Нативные артефакты в GitHub Releases
