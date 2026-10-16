//! GPU-обёртка на wgpu (Vulkan / Metal / DX12 / WebGPU).
//! GEMM (матричное умножение) на GPU через WGSL.
//! Работает на Intel / NVIDIA / AMD / Apple Silicon.
//! Fallback — в `gpu_cuda::матричное_умножение_mnk` (CUDA → CPU).

#[cfg(feature = "gpu")]
use wgpu::util::DeviceExt;

#[cfg(feature = "gpu")]
pub struct GpuWgpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,
}

#[cfg(feature = "gpu")]
impl GpuWgpu {
    /// Создать wgpu-контекст (async → sync через pollster).
    pub fn новый() -> Result<Self, String> {
        // На wasm32 pollster::block_on недоступен / нежелателен.
        #[cfg(target_arch = "wasm32")]
        {
            return Err("wgpu на wasm32: нужен async entry (WebGPU) — см. v7.7".into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let результат = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pollster::block_on(Self::новый_async())
            }));
            match результат {
                Ok(inner) => inner,
                Err(_) => Err("wgpu init panic".into()),
            }
        }
    }

    async fn новый_async() -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "Не найден GPU-адаптер (wgpu)".to_string())?;

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("ZIP VM GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("wgpu device: {e}"))?;

        Ok(Self {
            device,
            queue,
            adapter_info,
        })
    }

    /// Информация об адаптере (для отчёта / тестов).
    pub fn info(&self) -> String {
        format!(
            "{} ({:?}, {:?})",
            self.adapter_info.name, self.adapter_info.device_type, self.adapter_info.backend
        )
    }

    /// GEMM: C = A × B (N×N, f32, row-major).
    pub fn gemm(&self, a: &[f32], b: &[f32], n: usize) -> Result<Vec<f32>, String> {
        if a.len() != n * n || b.len() != n * n {
            return Err(format!(
                "shape: A={} B={} expect {}",
                a.len(),
                b.len(),
                n * n
            ));
        }
        if n == 0 {
            return Ok(vec![]);
        }

        let size = (n * n * std::mem::size_of::<f32>()) as u64;

        let buf_a = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("A"),
                contents: bytemuck::cast_slice(a),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_b = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("B"),
                contents: bytemuck::cast_slice(b),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_c = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("C"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // Uniform: 16-byte alignment (Vulkan).
        let params = [n as u32, 0u32, 0u32, 0u32];
        let buf_n = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("N"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("GEMM shader"),
                source: wgpu::ShaderSource::Wgsl(GEMM_SHADER.into()),
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("GEMM pipeline"),
                layout: None,
                module: &shader,
                entry_point: "main",
                compilation_options: Default::default(),
            });

        let bgl = pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GEMM bind group"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_c.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_n.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GEMM"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("GEMM pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let wg = 16usize;
            let dispatch = ((n + wg - 1) / wg) as u32;
            cpass.dispatch_workgroups(dispatch, dispatch, 1);
        }
        self.queue.submit(Some(encoder.finish()));

        let buf_result = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("C readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("readback"),
            });
        encoder.copy_buffer_to_buffer(&buf_c, 0, &buf_result, 0, size);
        self.queue.submit(Some(encoder.finish()));

        let slice = buf_result.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|e| format!("wgpu map channel: {e}"))?
            .map_err(|e| format!("wgpu map: {e}"))?;

        let data = slice.get_mapped_range();
        let результат: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        buf_result.unmap();

        Ok(результат)
    }
}

/// WGSL-шейдер GEMM (N×N, f32).
#[cfg(feature = "gpu")]
const GEMM_SHADER: &str = r#"
struct Params {
    n: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> c: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = params.n;
    let row = gid.y;
    let col = gid.x;
    if (row >= n || col >= n) {
        return;
    }

    var sum: f32 = 0.0;
    for (var k: u32 = 0u; k < n; k = k + 1u) {
        sum = sum + a[row * n + k] * b[k * n + col];
    }
    c[row * n + col] = sum;
}
"#;

// ─── Заглушка без feature gpu ───────────────────────────────────────────────

#[cfg(not(feature = "gpu"))]
pub struct GpuWgpu;

#[cfg(not(feature = "gpu"))]
impl GpuWgpu {
    pub fn новый() -> Result<Self, String> {
        Err("wgpu не скомпилирован (feature `gpu` отключён)".into())
    }

    pub fn gemm(&self, _a: &[f32], _b: &[f32], _n: usize) -> Result<Vec<f32>, String> {
        Err("wgpu не доступен".into())
    }

    pub fn info(&self) -> String {
        "wgpu не доступен".into()
    }
}
