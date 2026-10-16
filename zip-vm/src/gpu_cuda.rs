//! CUDA-обёртка для ZIP-VM.
//! GEMM (матричное умножение) на GPU через NVRTC.
//! Без feature `cuda` или без NVIDIA GPU — CPU blocked GEMM.

/// Есть ли любой GPU-бэкенд (wgpu или CUDA).
pub fn gpu_доступен() -> bool {
    #[cfg(feature = "gpu")]
    {
        if crate::gpu_wgpu::GpuWgpu::новый().is_ok() {
            return true;
        }
    }
    #[cfg(feature = "cuda")]
    {
        if GpuCuda::новый().is_ok() {
            return true;
        }
    }
    false
}

/// GEMM: C = A × B для квадратных N×N (f32).
/// Приоритет: wgpu → CUDA → CPU.
pub fn матричное_умножение(a: &[f32], b: &[f32], n: usize) -> Vec<f32> {
    матричное_умножение_mnk(a, b, n, n, n)
}

/// GEMM: C[M×N] = A[M×K] × B[K×N] (row-major, f32).
/// Для прямоугольных пока CUDA/CPU; wgpu — квадратные (m=k=n).
pub fn матричное_умножение_mnk(a: &[f32], b: &[f32], m: usize, k: usize, n: usize) -> Vec<f32> {
    // Приоритет 1: wgpu (Vulkan/Metal/DX12/WebGPU) — квадратные
    #[cfg(feature = "gpu")]
    {
        if m == k && k == n {
            let попытка = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let gpu = crate::gpu_wgpu::GpuWgpu::новый()?;
                gpu.gemm(a, b, n)
            }));
            if let Ok(Ok(результат)) = попытка {
                return результат;
            }
        }
    }

    // Приоритет 2: CUDA (NVIDIA)
    #[cfg(feature = "cuda")]
    {
        let попытка = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let gpu = GpuCuda::новый()?;
            gpu.gemm_mnk(a, b, m, k, n)
        }));
        if let Ok(Ok(результат)) = попытка {
            return результат;
        }
    }

    // Fallback: CPU
    gemm_cpu_blocked(a, b, m, k, n)
}

/// CPU fallback: blocked GEMM (ikj / block=64).
pub fn gemm_cpu_blocked(a: &[f32], b: &[f32], m: usize, k: usize, n: usize) -> Vec<f32> {
    assert_eq!(a.len(), m * k, "A size");
    assert_eq!(b.len(), k * n, "B size");
    let mut c = vec![0.0f32; m * n];
    let block = 64usize;
    for i0 in (0..m).step_by(block) {
        for j0 in (0..n).step_by(block) {
            for k0 in (0..k).step_by(block) {
                let i1 = (i0 + block).min(m);
                let j1 = (j0 + block).min(n);
                let k1 = (k0 + block).min(k);
                for i in i0..i1 {
                    for kk in k0..k1 {
                        let av = a[i * k + kk];
                        let brow = &b[kk * n + j0..kk * n + j1];
                        let crow = &mut c[i * n + j0..i * n + j1];
                        for (cj, &bj) in crow.iter_mut().zip(brow.iter()) {
                            *cj += av * bj;
                        }
                    }
                }
            }
        }
    }
    c
}

// ─── CUDA ───────────────────────────────────────────────────────────────────

#[cfg(feature = "cuda")]
use std::sync::Arc;
#[cfg(feature = "cuda")]
use cudarc::driver::{CudaContext, CudaStream, LaunchConfig, PushKernelArg};
#[cfg(feature = "cuda")]
use cudarc::nvrtc::compile_ptx;

#[cfg(feature = "cuda")]
pub struct GpuCuda {
    ctx: Arc<CudaContext>,
    stream: Arc<CudaStream>,
}

#[cfg(feature = "cuda")]
impl GpuCuda {
    /// Создать CUDA-контекст на устройстве 0.
    ///
    /// cudarc при отсутствии `nvcuda.dll` / libcuda может **паниковать**
    /// (dynamic load). Ловим panic → Err, чтобы VM ушла на CPU.
    pub fn новый() -> Result<Self, String> {
        let результат = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let ctx = CudaContext::new(0).map_err(|e| format!("CUDA init: {e}"))?;
            let stream = ctx.default_stream();
            Ok::<Self, String>(Self { ctx, stream })
        }));
        match результат {
            Ok(inner) => inner,
            Err(payload) => {
                let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "CUDA dynamic load panic".into()
                };
                Err(format!("CUDA недоступен: {msg}"))
            }
        }
    }

    /// Квадратный GEMM N×N.
    pub fn gemm(&self, a: &[f32], b: &[f32], n: usize) -> Result<Vec<f32>, String> {
        self.gemm_mnk(a, b, n, n, n)
    }

    /// Прямоугольный GEMM M×K × K×N.
    pub fn gemm_mnk(
        &self,
        a: &[f32],
        b: &[f32],
        m: usize,
        k: usize,
        n: usize,
    ) -> Result<Vec<f32>, String> {
        if a.len() != m * k || b.len() != k * n {
            return Err(format!(
                "shape: A={} expect {}, B={} expect {}",
                a.len(),
                m * k,
                b.len(),
                k * n
            ));
        }
        if m == 0 || k == 0 || n == 0 {
            return Ok(vec![]);
        }

        let d_a = self
            .stream
            .clone_htod(a)
            .map_err(|e| format!("htod A: {e}"))?;
        let d_b = self
            .stream
            .clone_htod(b)
            .map_err(|e| format!("htod B: {e}"))?;
        let mut d_c = self
            .stream
            .alloc_zeros::<f32>(m * n)
            .map_err(|e| format!("alloc C: {e}"))?;

        let ptx = compile_ptx(GEMM_KERNEL).map_err(|e| format!("nvrtc: {e}"))?;
        let module = self
            .ctx
            .load_module(ptx)
            .map_err(|e| format!("load module: {e}"))?;
        let kernel = module
            .load_function("matmul")
            .map_err(|e| format!("load fn: {e}"))?;

        let cfg = LaunchConfig {
            grid_dim: (
                ((n + 15) / 16) as u32,
                ((m + 15) / 16) as u32,
                1,
            ),
            block_dim: (16, 16, 1),
            shared_mem_bytes: 0,
        };

        let m_i = m as i32;
        let k_i = k as i32;
        let n_i = n as i32;
        let mut builder = self.stream.launch_builder(&kernel);
        builder.arg(&mut d_c);
        builder.arg(&d_a);
        builder.arg(&d_b);
        builder.arg(&m_i);
        builder.arg(&k_i);
        builder.arg(&n_i);
        unsafe {
            builder
                .launch(cfg)
                .map_err(|e| format!("launch: {e}"))?;
        }

        self.stream
            .synchronize()
            .map_err(|e| format!("sync: {e}"))?;
        self.stream
            .clone_dtoh(&d_c)
            .map_err(|e| format!("dtoh C: {e}"))
    }
}

/// CUDA-ядро GEMM (row-major f32). Один поток = один элемент C.
#[cfg(feature = "cuda")]
const GEMM_KERNEL: &str = r#"
extern "C" __global__ void matmul(
    float* C,
    const float* A,
    const float* B,
    int M,
    int K,
    int N
) {
    int ROW = blockIdx.y * blockDim.y + threadIdx.y;
    int COL = blockIdx.x * blockDim.x + threadIdx.x;
    if (ROW < M && COL < N) {
        float sum = 0.0f;
        for (int kk = 0; kk < K; kk++) {
            sum += A[ROW * K + kk] * B[kk * N + COL];
        }
        C[ROW * N + COL] = sum;
    }
}
"#;

// ─── Заглушка без feature cuda ──────────────────────────────────────────────

#[cfg(not(feature = "cuda"))]
pub struct GpuCuda;

#[cfg(not(feature = "cuda"))]
impl GpuCuda {
    pub fn новый() -> Result<Self, String> {
        Err("CUDA не скомпилирован (feature `cuda` отключён)".into())
    }

    pub fn gemm(&self, _a: &[f32], _b: &[f32], _n: usize) -> Result<Vec<f32>, String> {
        Err("CUDA не доступен".into())
    }

    pub fn gemm_mnk(
        &self,
        _a: &[f32],
        _b: &[f32],
        _m: usize,
        _k: usize,
        _n: usize,
    ) -> Result<Vec<f32>, String> {
        Err("CUDA не доступен".into())
    }
}

#[cfg(test)]
mod unit_cpu {
    use super::*;

    #[test]
    fn cpu_gemm_ones() {
        let n = 8;
        let a = vec![1.0f32; n * n];
        let b = vec![2.0f32; n * n];
        let c = gemm_cpu_blocked(&a, &b, n, n, n);
        assert!((c[0] - 2.0 * n as f32).abs() < 1e-4);
    }
}
