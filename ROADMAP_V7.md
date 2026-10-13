# План v7.0.0 — Кросс-компиляция

**Цель:** ZIP работает на всех платформах.

**Срок:** 2028

**Ключевой факт:** после v6.1.0 компилятор ZIP написан на самом ZIP (fixed-point v2 = v3 = v4). Для кросс-компиляции нужно **портировать только VM-интерпретатор** (Rust), а не весь компилятор.

## v7.0.0-alpha (2026-10-13) — ARM64

- [x] ARM64 Linux (`aarch64-unknown-linux-gnu`) + QEMU CI ✅
- [x] ARM64 macOS target (`aarch64-apple-darwin`) ✅
- [x] ARM64 Windows target (`aarch64-pc-windows-msvc`) ✅
- [x] Флаг `zip-vm --target …` (информационный) ✅
- [x] Документация [docs/ARM64.md](docs/ARM64.md) ✅
- [ ] RISC-V (следующий)
- [ ] WASM
- [ ] Нативные артефакты в GitHub Releases (все ARM64)

## Что нужно сделать дальше

### 1. RISC-V

**Платформы:** IoT, встраиваемые, education.

```bash
rustup target add riscv64gc-unknown-linux-gnu
cargo build --release --target riscv64gc-unknown-linux-gnu --manifest-path zip-vm/Cargo.toml
qemu-riscv64 ./zip-vm/target/riscv64gc-unknown-linux-gnu/release/zip-vm программа.zipb
```

### 2. WASM

**Платформы:** Браузер, edge computing.

```bash
wasm-pack build --target web  # или отдельная WASM-сборка VM
```

### 3. Нативные релизы ARM64

- Бинарники `zip-vm` для Linux/macOS/Windows ARM64 в Releases
- MSI/DEB/DMG под ARM64

## Результат v7.0.0

| Платформа | Статус |
|-----------|--------|
| x86-64 Linux | ✅ |
| x86-64 Windows | ✅ |
| x86-64 macOS | ✅ |
| ARM64 Linux | ✅ (v7.0.0-alpha, QEMU CI) |
| ARM64 macOS | ✅ (target) |
| ARM64 Windows | ✅ (target) |
| RISC-V | ⏳ |
| WASM | ⏳ |

**ZIP-байткод один на всех платформах.** См. [docs/ARM64.md](docs/ARM64.md).
