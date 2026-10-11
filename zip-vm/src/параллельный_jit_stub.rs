//! Заглушка параллельного JIT для wasm32.

use crate::байткод::Функция;
use crate::jit::JitFn;

pub struct ПараллельныйJIT;

impl ПараллельныйJIT {
    pub fn новый(_потоки: usize) -> Self {
        Self
    }

    pub fn из_окружения() -> Self {
        Self
    }

    pub fn скомпилировать_пакет(
        &mut self,
        задачи: Vec<(u32, Функция)>,
    ) -> Vec<(u32, Result<JitFn, String>)> {
        задачи
            .into_iter()
            .map(|(ид, _)| (ид, Err("JIT недоступен на wasm32".into())))
            .collect()
    }
}
