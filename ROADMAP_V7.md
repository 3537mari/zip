# План v7.0.0 — Кросс-компиляция

**Цель:** ZIP работает на всех платформах.

**Срок:** 2028

**Ключевой факт:** после v6.1.0 компилятор ZIP написан на самом ZIP (fixed-point v2 = v3 = v4). Для кросс-компиляции нужно **портировать только VM-интерпретатор** (Rust), а не весь компилятор.

## Что нужно сделать

### 1. ARM64

**Платформы:** Raspberry Pi, Apple Silicon, серверы ARM.

**Что нужно:**
- `cargo build --target aarch64-unknown-linux-gnu` для VM
- Тесты через QEMU (`qemu-aarch64`)
- ZIP-байткод — платформонезависим, не меняется
- Опционально: LLVM-бэкенд под ARM64 для `.exe`

```bash
zip-vm --target aarch64-unknown-linux-gnu программа.zipb
```

### 2. RISC-V

**Платформы:** IoT, встраиваемые, education.

**Что нужно:**
- `cargo build --target riscv64gc-unknown-linux-gnu` для VM
- Тесты через QEMU (`qemu-riscv64`)
- Поддержка RV32I, RV64I

```bash
zip-vm --target riscv64gc-unknown-linux-gnu программа.zipb
```

### 3. WASM

**Платформы:** Браузер, edge computing.

**Что нужно:**
- `wasm-pack build --target web` для VM (или отдельная WASM-сборка)
- Работа с памятью WASM (линейная память)
- Интеграция с JS (плейграунд)
- Уже есть `zipc.js` в `веб-сайт/wasm/` — обновить

```bash
zip-vm --target wasm32-unknown-unknown программа.zipb
```

### 4. Windows ARM

**Платформы:** Surface Pro X, новые Windows-устройства.

**Что нужно:**
- `cargo build --target aarch64-pc-windows-msvc` для VM
- Интеграция с Windows API
- Упаковка в `.exe` (опционально — через LLVM)

## Результат v7.0.0

| Платформа | Статус |
|-----------|--------|
| x86-64 Linux | ✅ |
| x86-64 Windows | ✅ |
| x86-64 macOS | ✅ |
| ARM64 Linux | ⏳ |
| ARM64 macOS | ⏳ |
| ARM64 Windows | ⏳ |
| RISC-V | ⏳ |
| WASM | ⏳ |

**ZIP работает везде.**
