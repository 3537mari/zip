# План v5.0.0 — Кросс-компиляция и полный ZIP-порт

**Цель:** ZIP на всех платформах; алгоритмы bootstrap/VM/STD без C/Rust-движка.

**Срок:** 2028

## Что нужно сделать

### 1. Полный порт bootstrap
- Лексер / парсер / генератор ZIPB **целиком на ZIP** (без `zip_self_compile` → C)
- Fixed-point без внешнего `bootstrap/zipc` на stage 1+

### 2. VM на ZIP
- Полный интерпретатор ZIPB в `zip-vm-zip/`

### 3. Кросс-компиляция
- ARM64, RISC-V, WASM

### 4. Мобильные биндинги
- Android (NDK + JNI), iOS

### 5. GPU / экосистема
- CUDA / Vulkan / Metal; пакетный реестр; IDE-плагины

## Результат v5.0.0

| Платформа | Цель |
|-----------|------|
| Windows / Linux / macOS x64 | ✅ |
| Linux / macOS ARM64 | ✅ |
| WASM | ✅ |
| Android / iOS | ✅ |
| RISC-V | ✅ |

**ZIP работает везде; bootstrap без C на stage 1+.**
