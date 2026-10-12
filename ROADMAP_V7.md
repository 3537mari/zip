# План v7.0.0 — Кросс-компиляция

**Цель:** ZIP на всех платформах.

**Срок:** 2028

## Что нужно сделать

### 1. ARM64

**Платформы:** Raspberry Pi, Apple Silicon, серверы ARM

**Что нужно:**
- Кодогенератор ARM64
- Ассемблер ARM64
- Оптимизации под ARM (NEON)
- Кросс-компиляция через clang

```bash
zip-vm --target aarch64-unknown-linux-gnu --gen программа.zip
```

### 2. RISC-V

**Платформы:** IoT, встраиваемые, education

```bash
zip-vm --target riscv64-unknown-elf --gen программа.zip
```

### 3. WASM

**Платформы:** Браузер, edge computing

```bash
zip-vm --target wasm32-unknown-unknown --gen программа.zip
```

### 4. Windows ARM

**Платформы:** Surface Pro X, новые Windows-устройства

## Результат v7.0.0

| Платформа | Статус |
|-----------|--------|
| x86-64 Linux / Windows / macOS | ✅ (есть) |
| ARM64 Linux / macOS / Windows | план |
| RISC-V | план |
| WASM | план |
