//! Библиотека виртуальной машины ZIP.

// Кириллические идентификаторы рядом с ASCII — норма для ZIP.
#![allow(confusable_idents)]

#[path = "байткод.rs"]
pub mod байткод;
#[path = "байткод_v3.rs"]
pub mod байткод_v3;
#[path = "байткод_v3_декодер.rs"]
pub mod байткод_v3_декодер;
#[path = "байткод_в_llvm.rs"]
pub mod байткод_в_llvm;
#[path = "ir_в_байткод.rs"]
pub mod ir_в_байткод;
#[path = "встроенные_v3.rs"]
pub mod встроенные_v3;
#[path = "zip_std_v4.rs"]
mod zip_std_v4;
#[path = "zip_self_host.rs"]
pub mod zip_self_host;
#[path = "zip_lex_полный.rs"]
pub mod zip_lex_полный;
#[path = "zip_ast.rs"]
pub mod zip_ast;
#[path = "zip_parse_полный.rs"]
pub mod zip_parse_полный;
#[path = "zip_gen_полный.rs"]
pub mod zip_gen_полный;
#[path = "загрузчик.rs"]
pub mod загрузчик;
#[path = "исполнитель.rs"]
pub mod исполнитель;
#[path = "исполнитель_v3.rs"]
pub mod исполнитель_v3;
#[cfg(not(target_arch = "wasm32"))]
#[path = "jit.rs"]
pub mod jit;
#[cfg(target_arch = "wasm32")]
#[path = "jit_stub.rs"]
pub mod jit;

#[cfg(not(target_arch = "wasm32"))]
#[path = "параллельный_jit.rs"]
pub mod параллельный_jit;
#[cfg(target_arch = "wasm32")]
#[path = "параллельный_jit_stub.rs"]
pub mod параллельный_jit;
#[path = "память.rs"]
pub mod память;
#[path = "планировщик.rs"]
pub mod планировщик;
#[path = "стек.rs"]
pub mod стек;

pub use байткод::{Значение, Инструкция, Модуль, Функция, ОшибкаVM, ВЕРСИЯ, МАГИЯ};
pub use байткод_v3::{Кодировщик, Опкод, МАГИЯ_V3, ВЕРСИЯ_V3};
pub use байткод_v3_декодер::{декодировать_код, разобрать_v3, это_zipb};
pub use байткод_в_llvm::байткод_в_llvm_ir;
pub use загрузчик::{загрузить, разобрать, сериализовать};
pub use исполнитель::Исполнитель;
pub use исполнитель_v3::{ИсполнительV3, ЗначениеV3, ОшибкаV3};
pub use jit::JitКомпилятор;
pub use параллельный_jit::ПараллельныйJIT;
