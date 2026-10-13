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

## v7.2.0 (2026-10-13) — ✅ iOS

- [x] **iOS (Swift + Objective-C bridge)** — `libzip_vm_ios.a` / `ZipVM.xcframework`
- [x] Targets: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios`
- [x] C-API `zip.h` + Swift `Zip.compile` / `run` / `compileAndRun`
- [x] CI [`.github/workflows/ios.yml`](.github/workflows/ios.yml)
- [x] Docs [docs/IOS.md](docs/IOS.md)

## v7.3.0 — Криптография

- [ ] SHA-256 на чистом ZIP (уже есть)
- [ ] AES-256-GCM на чистом ZIP
- [ ] Kyber-1024 на чистом ZIP
- [ ] Dilithium-5 на чистом ZIP

## v7.4.0 — GPU

- [ ] CUDA
- [ ] ROCm
- [ ] Metal
- [ ] Vulkan
