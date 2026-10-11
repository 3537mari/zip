//! Заглушка JIT для wasm32 (без libloading / clang).

use crate::байткод::Функция;
use std::collections::HashMap;

pub type JitFn = unsafe extern "C" fn(i64, i64, i64, i64) -> i64;

pub struct JitКомпилятор {
    функции: HashMap<u32, JitFn>,
}

impl JitКомпилятор {
    pub fn новый() -> Self {
        Self {
            функции: HashMap::new(),
        }
    }

    pub fn доступен() -> bool {
        false
    }

    pub fn получить(&self, id: u32) -> Option<JitFn> {
        self.функции.get(&id).copied()
    }

    pub fn скомпилировать_функцию(
        &mut self,
        _id: u32,
        _функция: &Функция,
    ) -> Result<JitFn, String> {
        Err("JIT недоступен на wasm32".into())
    }
}
