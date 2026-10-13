# Дорожная карта ZIP

## v1.0.0 – v3.0.0 ✅

См. [CHANGELOG.md](CHANGELOG.md).

## v4.0.0 (2026-10-12) — ✅ Runtime / STD / selfc-конвейер

- См. [docs/ОТЧЁТ-v4.md](docs/ОТЧЁТ-v4.md)

## v5.0.0 (2026-10-12) — ✅ Конвейер в VM

- См. [docs/ОТЧЁТ-v5.md](docs/ОТЧЁТ-v5.md)

## v6.0.0 (2026-10-12) — ✅ ЗАВЕРШЁН

- [x] Structs в ZIP ✅
- [x] Массивы в ZIP ✅
- [x] Указатели в ZIP ✅
- [x] Компилятор-драйвер на чистом ZIP (fixed-point v2 = v3) ✅

**Дата:** 2026-10-12 · [docs/ОТЧЁТ-v6.md](docs/ОТЧЁТ-v6.md) · [ROADMAP_V6.md](ROADMAP_V6.md)

## v6.1.0 (2026-10-12) — ✅ ЗАВЕРШЁН

- [x] Полный lowering лексера в ZIP-байткод ✅
- [x] Полный lowering парсера в ZIP-байткод ✅
- [x] Полный lowering генератора ZIPB в ZIP-байткод ✅
- [x] Убраны Rust-функции компилятора (`zip_self_lex` / `zip_self_parse` / `zip_self_gen`) ✅
- [x] Fixed-point без Rust-алгоритмов: v2 = v3 = v4 = `03ec79b9...` ✅

**Релиз:** https://github.com/3537mari/zip/releases/tag/v6.1.0

## v7.0.0-alpha (2026-10-13) — ✅ ARM64

- [x] ARM64 Linux + QEMU CI ✅
- [x] ARM64 macOS / Windows targets ✅
- [x] Документация [docs/ARM64.md](docs/ARM64.md) ✅

## v7.0.0-beta (2026-10-13) — ✅ RISC-V

- [x] RISC-V 64 (`riscv64gc-unknown-linux-gnu`) + QEMU CI ✅
- [x] Документация [docs/RISCV.md](docs/RISCV.md) ✅

## v7.0.0-rc (2026-10-13) — ✅ WASM

- [x] WASM VM + плейграунд (compile + run) ✅
- [x] Документация [docs/WASM.md](docs/WASM.md) ✅

## v7.0.0 (2026-10-13) — ✅ Кросс-компиляция (финал)

**Важно:** начиная с v6.1.0 кросс-компиляция — это **задача бэкенда (Rust-VM)**, а не порта всего компилятора.

- [x] ARM64 Linux / macOS — alpha ✅
- [x] RISC-V 64 — beta ✅
- [x] WASM (браузер) — rc ✅
- [x] **Windows ARM64** — v7.0.0 ✅
- [ ] Нативные пакеты ARM64/RISC-V в Releases (post-v7)

См. [ROADMAP_V7.md](ROADMAP_V7.md), [docs/WINDOWS-ARM.md](docs/WINDOWS-ARM.md).

## v8.0.0 (2028) — Мобильные биндинги

- [ ] Android (NDK + JNI)
- [ ] iOS (Swift + Objective-C bridge)

## v9.0.0 (2029) — GPU-вычисления

- [ ] CUDA (NVIDIA)
- [ ] ROCm (AMD)
- [ ] Metal (Apple)
- [ ] Vulkan Compute

## v10.0.0 (2029) — Пакетный реестр

- [ ] Облако + CDN (zip-lang.org)
- [ ] GitHub OAuth
- [ ] 100+ пакетов
- [ ] CDN для быстрой загрузки
