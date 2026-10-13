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

## v7.0.0 (2028) — Кросс-компиляция

**Важно:** начиная с v6.1.0 кросс-компиляция — это **задача бэкенда (Rust-VM)**, а не порта всего компилятора.

- [ ] ARM64 (Linux, macOS, Windows)
- [ ] RISC-V (RV32I, RV64I)
- [ ] WASM (браузер, edge)
- [ ] Windows ARM

См. [ROADMAP_V7.md](ROADMAP_V7.md).

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
