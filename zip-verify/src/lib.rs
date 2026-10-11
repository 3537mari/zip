//! Библиотека статической верификации ZIPB (типы, владение, PERFECT).

#![allow(confusable_idents)]

#[path = "типы.rs"]
pub mod типы;
#[path = "владение.rs"]
pub mod владение;
#[path = "perfect.rs"]
pub mod perfect;

pub use perfect::{проверить_perfect, ОшибкаPerfect};
pub use владение::{проверить_владение, ОшибкаВладения};
pub use типы::{проверить_инструкции, проверить_функцию, ОшибкаВерификации, Тип};
