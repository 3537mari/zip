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

## v7.0.0-beta (2026-10-13) — RISC-V

- [x] RISC-V 64 (`riscv64gc-unknown-linux-gnu`) + QEMU CI ✅
- [x] Документация [docs/RISCV.md](docs/RISCV.md) ✅
- [ ] WASM (следующий)
- [ ] Нативные артефакты в GitHub Releases (ARM64 / RISC-V)

## Что нужно сделать дальше

### 1. WASM

**Платформы:** Браузер, edge computing.

```bash
wasm-pack build --target web  # или отдельная WASM-сборка VM
```

### 2. Нативные релизы

- Бинарники `zip-vm` для Linux ARM64 / RISC-V в Releases
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
| RISC-V 64 | ✅ (v7.0.0-beta, QEMU CI) |
| WASM | ⏳ |

**ZIP-байткод один на всех платформах.** См. [docs/ARM64.md](docs/ARM64.md), [docs/RISCV.md](docs/RISCV.md).
