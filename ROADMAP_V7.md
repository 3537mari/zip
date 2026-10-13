# План v7 — Кросс-компиляция и мобильные

**Ключевой факт:** после v6.1.0 компилятор ZIP написан на самом ZIP. Портируется только VM.

## v7.0.0 (2026-10-13) — ✅ Кросс-компиляция

- [x] ARM64 Linux / macOS / Windows
- [x] RISC-V 64
- [x] WASM (браузер)
- [x] Документация ARM64 / RISCV / WASM / WINDOWS-ARM

## v7.1.0 (2026-10-13) — ✅ Android

- [x] **Android (NDK + JNI)** — `libzip_vm_android.so`
- [x] Kotlin API `org.ziplang.zip.Zip`
- [x] CI [`.github/workflows/android.yml`](.github/workflows/android.yml)
- [x] Docs [docs/ANDROID.md](docs/ANDROID.md)
- [ ] iOS (Swift + FFI) — следующий

## v7.2+ — дальше

- [ ] Криптография на ZIP (AES / Kyber / Dilithium)
- [ ] GPU (CUDA / ROCm / Metal / Vulkan)
- [ ] Пакетный реестр
