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

## v7.0.0-rc (2026-10-13) — WASM

- [x] **WASM (браузер)** ✅ — `wasm32-unknown-unknown` + wasm-bindgen
- [x] Плейграунд: компилировать и запускать ZIP в браузере
- [x] CI: [`.github/workflows/wasm.yml`](.github/workflows/wasm.yml) + headless Chrome
- [x] Документация [docs/WASM.md](docs/WASM.md) ✅
- [ ] Нативные артефакты в GitHub Releases (ARM64 / RISC-V)
- [ ] Windows ARM (нативная сборка) — следующий

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
| WASM | ✅ (v7.0.0-rc, браузер) |

**ZIP-байткод один на всех платформах.** См. [docs/ARM64.md](docs/ARM64.md), [docs/RISCV.md](docs/RISCV.md), [docs/WASM.md](docs/WASM.md).
