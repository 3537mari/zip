//! WASM-биндинги для онлайн-компилятора ZIP (плейграунд).

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

/// Разбор исходника: лексика → AST → IR (без нативной линковки).
#[wasm_bindgen]
pub fn скомпилировать(исходник: &str) -> String {
    let mut лексер = crate::Лексер::новый(исходник);
    let токены = match лексер.разобрать() {
        Ok(т) => т,
        Err(е) => return format!("Ошибка лексики: {}", е),
    };

    let дерево = match crate::разобрать_исходник(исходник) {
        Ok(д) => д,
        Err(е) => return format!("Ошибка парсинга: {}", е),
    };

    let функции = match crate::сгенерировать_из_исходника(исходник) {
        Ok(ф) => ф,
        Err(е) => return format!("Ошибка IR: {}", е),
    };

    let llvm = match crate::llvm_ir::сгенерировать_llvm_ir(&функции, "playground") {
        Ok(ир) => ир,
        Err(е) => format!("(LLVM IR недоступен: {})", е),
    };

    let _ = &дерево;
    format!(
        "✓ Компиляция успешна\n\nТокенов: {}\nФункций: {}\n\n=== LLVM IR ===\n{}",
        токены.len(),
        функции.len(),
        llvm
    )
}

#[wasm_bindgen]
pub fn версия() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
