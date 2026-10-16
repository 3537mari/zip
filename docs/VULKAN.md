# GPU-вычисления в ZIP (Vulkan / wgpu)

## Что нужно знать

[wgpu](https://wgpu.rs) — кросс-платформенная абстракция над **Vulkan**, **Metal**, **DirectX 12**, **WebGPU**. Один код ZIP-VM работает на:

- Intel (UHD, Arc)
- NVIDIA (GTX, RTX)
- AMD (Radeon)
- Apple Silicon (M1–M4)
- Браузер (WebGPU) — планируется v7.7

CUDA (`feature cuda`) остаётся опциональным ускорителем для NVIDIA.

## Установка

### Windows

```bash
winget install KhronosGroup.VulkanSDK
```

Проверка:

```bash
vulkaninfo --summary
# GPU0: deviceName = Intel(R) UHD Graphics / NVIDIA / AMD
```

### Linux

```bash
sudo apt install libvulkan-dev mesa-vulkan-drivers vulkan-tools
```

### macOS

Metal backend встроен в wgpu (отдельный SDK не нужен).

## Сборка

```bash
cd zip-vm
cargo build --release --features gpu
```

Принудительно Vulkan:

```bash
# PowerShell
$env:WGPU_BACKEND = "vulkan"
cargo test --features gpu -- --nocapture
```

## Использование

```zip
#runtime("io")

функция главная() {
    n = 512
    a = создать_тензор(n, n, 0, 0)
    б = создать_тензор(n, n, 0, 0)
    заполнить_тензор(a, 1.0)
    заполнить_тензор(б, 2.0)
    // Автоматически: wgpu → CUDA → CPU
    в = матричное_умножение(a, б, n, n, n)
    возврат(0)
}
```

## Архитектура

```
ZIP-программа
    ↓ матричное_умножение
VM (zip_std_gemm)
    ↓ приоритет 1: wgpu
GpuWgpu (gpu_wgpu.rs) + WGSL
    ↓ Vulkan / Metal / DX12 / WebGPU
Intel / NVIDIA / AMD / Apple GPU
    ↓ приоритет 2: CUDA (feature cuda)
    ↓ приоритет 3: CPU blocked GEMM
```

## Fallback

1. **wgpu** (`feature gpu`) — если адаптер найден  
2. **CUDA** (`feature cuda`) — если NVIDIA + драйвер  
3. **CPU** blocked GEMM — всегда

## Проверено

| Платформа | Backend | Статус |
|-----------|---------|--------|
| Intel UHD Graphics (Vulkan 1.2.177) | wgpu → Vulkan | ✅ |
| CPU fallback (без `gpu`) | — | ✅ |
| NVIDIA CUDA | `feature cuda` | ✅ (v7.5) |

## Производительность (ориентир)

| GPU | 512×512 | 1024×1024 |
|-----|---------|-----------|
| Intel UHD | ~15–50 мс | ~120+ мс |
| NVIDIA RTX | ~3 мс | ~15 мс |
| CPU blocked | ~30–300 мс* | ~250+ мс |

\* зависит от `debug`/`release` и CPU.

## Тесты

```bash
cargo test --manifest-path zip-vm/Cargo.toml --features gpu --test тест_wgpu -- --nocapture
```

## Что дальше

- WebGPU в браузерном плейграунде (v7.7)
- Tiled GEMM для Intel UHD
- Прямоугольный GEMM (M×K×N) на WGSL
