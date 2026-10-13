# План v7.0.0 — Кросс-компиляция

**Цель:** ZIP работает на всех платформах.

**Срок:** 2028 → **закрыт 2026-10-13**

**Ключевой факт:** после v6.1.0 компилятор ZIP написан на самом ZIP (fixed-point v2 = v3 = v4). Для кросс-компиляции нужно **портировать только VM-интерпретатор** (Rust), а не весь компилятор.

## v7.0.0-alpha (2026-10-13) — ARM64 Linux/macOS

- [x] ARM64 Linux (`aarch64-unknown-linux-gnu`) + QEMU CI ✅
- [x] ARM64 macOS target (`aarch64-apple-darwin`) ✅
- [x] Флаг `zip-vm --target …` (информационный) ✅
- [x] Документация [docs/ARM64.md](docs/ARM64.md) ✅

## v7.0.0-beta (2026-10-13) — RISC-V

- [x] RISC-V 64 (`riscv64gc-unknown-linux-gnu`) + QEMU CI ✅
- [x] Документация [docs/RISCV.md](docs/RISCV.md) ✅

## v7.0.0-rc (2026-10-13) — WASM

- [x] WASM (браузер) ✅ — `wasm32-unknown-unknown` + wasm-bindgen
- [x] Плейграунд: компилировать и запускать ZIP в браузере
- [x] CI + docs [docs/WASM.md](docs/WASM.md) ✅

## v7.0.0 (2026-10-13) — Windows ARM64 / финал

- [x] **ARM64 Windows** (`aarch64-pc-windows-msvc`) ✅
- [x] CI: [`.github/workflows/windows-arm.yml`](.github/workflows/windows-arm.yml) (`windows-11-arm` + cross)
- [x] Документация [docs/WINDOWS-ARM.md](docs/WINDOWS-ARM.md) ✅

**v7.0.0 полностью закрыт. Все целевые платформы поддержаны.**

## Результат v7.0.0

| Платформа | Статус |
|-----------|--------|
| x86-64 Linux | ✅ |
| x86-64 Windows | ✅ |
| x86-64 macOS | ✅ |
| ARM64 Linux | ✅ (QEMU CI) |
| ARM64 macOS | ✅ (target) |
| ARM64 Windows | ✅ (v7.0.0, `windows-11-arm` CI) |
| RISC-V 64 | ✅ (QEMU CI) |
| WASM | ✅ (браузер) |

**ZIP-байткод один на всех платформах.**

## v8.0.0 — Мобильные биндинги

- [ ] Android (NDK + JNI) — следующий
- [ ] iOS (Swift + Objective-C bridge)
