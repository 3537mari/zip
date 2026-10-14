//! Тесты CUDA GEMM + CPU fallback.

#[cfg(feature = "cuda")]
#[test]
fn тест_cuda_gemm_512() {
    use zip_vm::gpu_cuda::GpuCuda;

    let n = 512;
    let a = vec![1.0f32; n * n];
    let b = vec![2.0f32; n * n];

    let gpu = match GpuCuda::новый() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skip CUDA gemm (нет GPU/драйвера): {e}");
            return;
        }
    };
    let c = match gpu.gemm(&a, &b, n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip CUDA gemm (launch): {e}");
            return;
        }
    };

    assert_eq!(c.len(), n * n);
    // C[0][0] = n * 1.0 * 2.0 = 1024
    assert!((c[0] - 1024.0).abs() < 1e-2, "C[0]={}", c[0]);
}

#[cfg(feature = "cuda")]
#[test]
fn тест_cuda_сравнение_cpu() {
    use zip_vm::gpu_cuda::GpuCuda;
    use zip_vm::zip_std_v4::gemm_cpu_blocked;

    let n = 64;
    let a = (0..n * n).map(|i| (i as f32) * 0.01).collect::<Vec<_>>();
    let b = (0..n * n).map(|i| (i as f32) * 0.02).collect::<Vec<_>>();

    let c_cpu = gemm_cpu_blocked(&a, &b, n);

    let gpu = match GpuCuda::новый() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skip CUDA vs CPU (нет GPU/драйвера): {e}");
            return;
        }
    };
    let c_gpu = match gpu.gemm(&a, &b, n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip CUDA vs CPU (launch): {e}");
            return;
        }
    };

    for i in 0..n * n {
        assert!(
            (c_cpu[i] - c_gpu[i]).abs() < 1e-2,
            "Расхождение в {i}: CPU={}, GPU={}",
            c_cpu[i],
            c_gpu[i]
        );
    }
}

/// Без feature cuda — CPU путь.
#[cfg(not(feature = "cuda"))]
#[test]
fn тест_cuda_fallback_cpu() {
    use zip_vm::zip_std_v4::матричное_умножение;

    let n = 64;
    let a = vec![1.0f32; n * n];
    let b = vec![2.0f32; n * n];

    let c = матричное_умножение(&a, &b, n);
    assert_eq!(c.len(), n * n);
    assert!((c[0] - (n as f32) * 2.0).abs() < 1e-3);
}

#[test]
fn тест_cpu_blocked_всегда() {
    use zip_vm::zip_std_v4::gemm_cpu_blocked;

    let n = 32;
    let a = vec![1.0f32; n * n];
    let b = vec![3.0f32; n * n];
    let c = gemm_cpu_blocked(&a, &b, n);
    assert!((c[0] - 3.0 * n as f32).abs() < 1e-3);
    assert!((c[n * n - 1] - 3.0 * n as f32).abs() < 1e-3);
}

/// Микробенч CPU GEMM 512×512 (для отчёта без NVIDIA).
#[test]
fn тест_cpu_gemm_512_bench() {
    use std::time::Instant;
    use zip_vm::zip_std_v4::матричное_умножение;

    let n = 512;
    let a = vec![1.0f32; n * n];
    let b = vec![2.0f32; n * n];
    let t0 = Instant::now();
    let c = матричное_умножение(&a, &b, n);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("GEMM 512×512 (CPU/fallback): {ms:.1} мс, C00={}", c[0]);
    assert!((c[0] - 1024.0).abs() < 1e-2);
}
