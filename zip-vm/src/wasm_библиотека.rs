//! WASM-библиотека для плейграунда ZIP.
//! Компилирует и запускает ZIP-программы прямо в браузере.

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

use crate::байткод_v3_декодер::разобрать_v3;
use crate::исполнитель_v3::ИсполнительV3;
use crate::zip_gen_полный::сгенерировать_из_исходника;

/// Компилировать ZIP-исходник в ZIPB (hex).
#[wasm_bindgen(js_name = compileZip)]
pub fn скомпилировать(исходник: &str) -> String {
    match сгенерировать_из_исходника(исходник) {
        Ok(байткод) => hex_encode(&байткод),
        Err(e) => format!("Ошибка компиляции: {e}"),
    }
}

/// Запустить ZIPB-байткод (hex) в VM.
#[wasm_bindgen(js_name = runZip)]
pub fn запустить(байткод_hex: &str) -> String {
    let байткод = match hex_decode(байткод_hex.trim()) {
        Ok(b) => b,
        Err(e) => return format!("Ошибка декодирования: {e}"),
    };
    запустить_байты(&байткод)
}

/// Полная цепочка: исходник → вывод + результат.
#[wasm_bindgen(js_name = compileAndRun)]
pub fn скомпилировать_и_запустить(исходник: &str) -> String {
    let байткод = match сгенерировать_из_исходника(исходник) {
        Ok(b) => b,
        Err(e) => return format!("Ошибка компиляции: {e}"),
    };
    запустить_байты(&байткод)
}

/// Версия crate zip-vm.
#[wasm_bindgen(js_name = versionZip)]
pub fn версия() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn запустить_байты(байткод: &[u8]) -> String {
    let модуль = match разобрать_v3(байткод) {
        Ok(m) => m,
        Err(e) => return format!("Ошибка ZIPB: {e}"),
    };
    let mut vm = match ИсполнительV3::новый(модуль) {
        Ok(v) => v,
        Err(e) => return format!("Ошибка VM: {e}"),
    };
    match vm.запустить_с_выводом() {
        Ok(вывод) => вывод,
        Err(e) => format!("Ошибка VM: {e}"),
    }
}

fn hex_encode(данные: &[u8]) -> String {
    let mut s = String::with_capacity(данные.len() * 2);
    for b in данные {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("нечётная длина hex".into());
    }
    let mut результат = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let a = bytes[i] as char;
        let b = bytes[i + 1] as char;
        let байт = u8::from_str_radix(&format!("{a}{b}"), 16)
            .map_err(|e| format!("hex: {e}"))?;
        результат.push(байт);
        i += 2;
    }
    Ok(результат)
}
