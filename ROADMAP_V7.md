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

- [x] **iOS (Swift + Objective-C bridge)** — `ZipVM.xcframework`
- [x] CI [`.github/workflows/ios.yml`](.github/workflows/ios.yml)
- [x] Docs [docs/IOS.md](docs/IOS.md)

## v7.3.0 (2026-10-13) — ✅ Криптография на чистом стеке ZIP

- [x] SHA-256 / SHA-3 (без OpenSSL)
- [x] AES-256-GCM (NIST vectors)
- [x] Kyber-1024 / ML-KEM-1024 (KEM round-trip)
- [x] Dilithium-5 / ML-DSA-87 (подпись/проверка)
- [x] Constant-time сравнение
- [x] getrandom через OS CSPRNG
- [x] Интеграция `@шифровать` + ZIP STD
- [x] Docs [docs/КРИПТОГРАФИЯ.md](docs/КРИПТОГРАФИЯ.md)

**Никакой зависимости от OpenSSL в основном пути.**

## v7.4.0 (2026-10-13) — AES-256-GCM на чистом ZIP

- [x] AES-256 (S-box, key expansion, шифрование блоков)
- [x] GCM (CTR + GHASH)
- [x] Constant-time сравнение
- [x] NIST-вектора
- [x] Fixed-point сохранён
- [x] VM builtins `zip_crypto_aes_*` убраны

## v7.4.1 — Kyber-1024 на чистом ZIP

- [ ] ML-KEM-1024 (FIPS 203) на чистом ZIP

## v7.4.2 — Dilithium-5 на чистом ZIP

- [ ] ML-DSA-87 (FIPS 204) на чистом ZIP

## v7.5.0 — GPU

- [ ] CUDA
- [ ] ROCm
- [ ] Metal
- [ ] Vulkan
