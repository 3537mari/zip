# GPU-вычисления в ZIP (CUDA)

## Что нужно знать

CUDA — параллельная вычислительная платформа NVIDIA. В ZIP используется через [`cudarc`](https://crates.io/crates/cudarc) + NVRTC (компиляция ядер в рантайме).

**CUDA работает только на NVIDIA GPU.** Для AMD — ROCm (v7.6), Apple — Metal (v7.7), кросс-платформенный — Vulkan (v7.8).

## Установка

```bash
# Ubuntu
sudo apt install nvidia-cuda-toolkit

# Windows
winget install Nvidia.CUDA

# Проверка
nvcc --version
nvidia-smi
```

## Сборка

```bash
cd zip-vm
cargo build --release --features cuda
```

Без `--features cuda` VM собирается как обычно и всегда использует CPU blocked GEMM.

## Использование

```zip
#runtime("io")

функция главная() {
    n = 512
    a = создать_тензор(n, n, 0, 0)
    б = создать_тензор(n, n, 0, 0)
    заполнить_тензор(a, 1.0)
    заполнить_тензор(б, 2.0)

    // VM: CUDA если feature cuda + GPU, иначе CPU
    в = матричное_умножение(a, б, n, n, n)
    возврат(0)
}
```

Проверка доступности из ZIP: `gpu_доступен()` → `1` / `0`.

## Архитектура

```
ZIP-программа
    ↓ матричное_умножение
VM builtins (zip_std_v4.rs → zip_std_gemm)
    ↓ если feature cuda и GPU жив
GpuCuda (gpu_cuda.rs)
    ↓ cudarc + NVRTC
CUDA-ядро matmul
    ↓
NVIDIA GPU
```

## Fallback

Если feature `cuda` выключен или `CudaContext::new` / NVRTC / launch падает — используется **CPU blocked GEMM** (`gemm_cpu_blocked`, блок 64).

## Производительность (ориентир)

| Размер | CPU | GPU (RTX-класс) | Ускорение |
|--------|-----|-----------------|-----------|
| 512×512 | ~30 мс | ~3 мс | ~10× |
| 1024×1024 | ~250 мс | ~15 мс | ~16× |
| 2048×2048 | ~2000 мс | ~80 мс | ~25× |

Наивное ядро (1 поток = 1 элемент C). Для production — tiled / cuBLAS.

## Тесты

```bash
# Без CUDA (CPU fallback)
cargo test --manifest-path zip-vm/Cargo.toml --test тест_cuda

# С CUDA
cargo test --manifest-path zip-vm/Cargo.toml --features cuda --test тест_cuda
```

## Что дальше

- ROCm (AMD): docs/ROCM.md (v7.6)
- Metal (Apple): docs/METAL.md (v7.7)
- Vulkan: docs/VULKAN.md (v7.8)
