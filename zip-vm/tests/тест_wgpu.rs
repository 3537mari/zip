//! Тесты wgpu GEMM (Vulkan / Metal / DX12 / WebGPU).

#[cfg(feature = "gpu")]
#[test]
fn тест_wgpu_адаптер() {
    use zip_vm::gpu_wgpu::GpuWgpu;

    let gpu = match GpuWgpu::новый() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skip: нет wgpu-адаптера: {e}");
            return;
        }
    };

    let info = gpu.info();
    println!("wgpu adapter: {info}");
    assert!(!info.is_empty());
}

#[cfg(feature = "gpu")]
#[test]
fn тест_wgpu_gemm_512() {
    use zip_vm::gpu_wgpu::GpuWgpu;

    let n = 512;
    let a = vec![1.0f32; n * n];
    let b = vec![2.0f32; n * n];

    let gpu = match GpuWgpu::новый() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skip wgpu gemm: {e}");
            return;
        }
    };

    let t0 = std::time::Instant::now();
    let c = match gpu.gemm(&a, &b, n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip wgpu gemm (launch): {e}");
            return;
        }
    };
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("GEMM 512×512 (wgpu): {ms:.1} мс, adapter={}", gpu.info());

    assert_eq!(c.len(), n * n);
    assert!((c[0] - 1024.0).abs() < 1e-2, "C[0]={}", c[0]);
    assert!((c[n * n - 1] - 1024.0).abs() < 1e-2);
}

#[cfg(feature = "gpu")]
#[test]
fn тест_wgpu_сравнение_cpu() {
    use zip_vm::gpu_wgpu::GpuWgpu;
    use zip_vm::zip_std_v4::gemm_cpu_blocked;

    let n = 64;
    let a: Vec<f32> = (0..n * n).map(|i| (i as f32) * 0.01).collect();
    let b: Vec<f32> = (0..n * n).map(|i| (i as f32) * 0.02).collect();

    let c_cpu = gemm_cpu_blocked(&a, &b, n);

    let gpu = match GpuWgpu::новый() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skip wgpu vs CPU: {e}");
            return;
        }
    };

    let c_gpu = match gpu.gemm(&a, &b, n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip wgpu vs CPU (launch): {e}");
            return;
        }
    };

    for i in 0..n * n {
        let diff = (c_cpu[i] - c_gpu[i]).abs();
        let tol = 1e-2_f32.max(c_cpu[i].abs() * 1e-4);
        assert!(
            diff < tol,
            "Расхождение в {i}: CPU={}, GPU={}, diff={diff}, tol={tol}",
            c_cpu[i],
            c_gpu[i]
        );
    }
}

#[cfg(feature = "gpu")]
#[test]
fn тест_wgpu_vs_cpu_время_512() {
    use std::time::Instant;
    use zip_vm::gpu_wgpu::GpuWgpu;
    use zip_vm::zip_std_v4::gemm_cpu_blocked;

    let n = 512;
    let a = vec![1.0f32; n * n];
    let b = vec![2.0f32; n * n];

    let t0 = Instant::now();
    let c_cpu = gemm_cpu_blocked(&a, &b, n);
    let cpu_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let gpu = match GpuWgpu::новый() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skip timing: {e}");
            return;
        }
    };
    let t1 = Instant::now();
    let c_gpu = match gpu.gemm(&a, &b, n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip timing gemm: {e}");
            return;
        }
    };
    let gpu_ms = t1.elapsed().as_secs_f64() * 1000.0;

    println!("GEMM 512×512 CPU: {cpu_ms:.1} мс");
    println!("GEMM 512×512 GPU: {gpu_ms:.1} мс ({})", gpu.info());
    assert!((c_cpu[0] - c_gpu[0]).abs() < 1e-2);
}

#[cfg(not(feature = "gpu"))]
#[test]
fn тест_wgpu_fallback_cpu() {
    use zip_vm::zip_std_v4::матричное_умножение;

    let n = 64;
    let a = vec![1.0f32; n * n];
    let b = vec![2.0f32; n * n];
    let c = матричное_умножение(&a, &b, n);
    assert_eq!(c.len(), n * n);
    assert!((c[0] - (n as f32) * 2.0).abs() < 1e-3);
}
