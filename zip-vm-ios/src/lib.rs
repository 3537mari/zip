//! C-API ZIP-VM для iOS.
//! Экспортируется как static library (`libzip_vm_ios.a`) + заголовок `zip.h`.
//!
//! Имена C-экспортов — ASCII (Swift / Objective-C bridging).

use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

thread_local! {
    static LAST_OK: Cell<bool> = const { Cell::new(true) };
}

fn set_ok(ok: bool) {
    LAST_OK.with(|c| c.set(ok));
}

/// Компилировать ZIP-исходник в ZIPB (hex).
/// Возвращает hex-строку (caller освобождает через `zip_free`).
/// При ошибке — NULL.
#[no_mangle]
pub extern "C" fn zip_compile(source: *const c_char) -> *mut c_char {
    if source.is_null() {
        set_ok(false);
        return std::ptr::null_mut();
    }
    let s = unsafe { CStr::from_ptr(source) };
    let s = match s.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_ok(false);
            return std::ptr::null_mut();
        }
    };

    match zip_vm::zip_компилятор::скомпилировать(s) {
        Ok(байткод) => {
            let hex = hex_encode(&байткод);
            match CString::new(hex) {
                Ok(c) => {
                    set_ok(true);
                    c.into_raw()
                }
                Err(_) => {
                    set_ok(false);
                    std::ptr::null_mut()
                }
            }
        }
        Err(_) => {
            set_ok(false);
            std::ptr::null_mut()
        }
    }
}

/// Запустить ZIPB (hex), вернуть вывод.
#[no_mangle]
pub extern "C" fn zip_run(bytecode_hex: *const c_char) -> *mut c_char {
    if bytecode_hex.is_null() {
        set_ok(false);
        return std::ptr::null_mut();
    }
    let s = unsafe { CStr::from_ptr(bytecode_hex) };
    let hex = match s.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_ok(false);
            return std::ptr::null_mut();
        }
    };
    let байткод = match hex_decode(hex) {
        Ok(b) => b,
        Err(_) => {
            set_ok(false);
            return std::ptr::null_mut();
        }
    };

    match zip_vm::zip_компилятор::запустить(&байткод) {
        Ok(вывод) => match CString::new(вывод) {
            Ok(c) => {
                set_ok(true);
                c.into_raw()
            }
            Err(_) => {
                set_ok(false);
                std::ptr::null_mut()
            }
        },
        Err(_) => {
            set_ok(false);
            std::ptr::null_mut()
        }
    }
}

/// Полная цепочка: исходник → результат.
#[no_mangle]
pub extern "C" fn zip_compile_and_run(source: *const c_char) -> *mut c_char {
    if source.is_null() {
        set_ok(false);
        return std::ptr::null_mut();
    }
    let s = unsafe { CStr::from_ptr(source) };
    let s = match s.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_ok(false);
            return std::ptr::null_mut();
        }
    };

    match zip_vm::zip_компилятор::скомпилировать_и_запустить(s) {
        Ok(вывод) => match CString::new(вывод) {
            Ok(c) => {
                set_ok(true);
                c.into_raw()
            }
            Err(_) => {
                set_ok(false);
                std::ptr::null_mut()
            }
        },
        Err(_) => {
            set_ok(false);
            std::ptr::null_mut()
        }
    }
}

/// Освободить строку из `zip_compile` / `zip_run` / `zip_compile_and_run`.
#[no_mangle]
pub extern "C" fn zip_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

/// Версия ZIP (статическая строка, не освобождать).
#[no_mangle]
pub extern "C" fn zip_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// 1 — последняя операция успешна, 0 — иначе.
#[no_mangle]
pub extern "C" fn zip_last_success() -> c_int {
    if LAST_OK.with(|c| c.get()) {
        1
    } else {
        0
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
        let байт =
            u8::from_str_radix(&format!("{a}{b}"), 16).map_err(|e| format!("hex: {e}"))?;
        результат.push(байт);
        i += 2;
    }
    Ok(результат)
}

#[cfg(test)]
mod тесты {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn версия_не_пуста() {
        let v = unsafe { CStr::from_ptr(zip_version()) };
        assert_eq!(v.to_str().unwrap(), "7.2.0");
    }

    #[test]
    fn арифметика_через_c_api() {
        let src = CString::new("функция главная() { возврат(2 + 3) }").unwrap();
        let out_ptr = zip_compile_and_run(src.as_ptr());
        assert!(!out_ptr.is_null());
        assert_eq!(zip_last_success(), 1);
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_string_lossy().into_owned();
        zip_free(out_ptr);
        assert!(out.contains("5"), "{out}");
    }

    #[test]
    fn compile_run_hex() {
        let src = CString::new("функция главная() { возврат(7) }").unwrap();
        let hex_ptr = zip_compile(src.as_ptr());
        assert!(!hex_ptr.is_null());
        let hex = unsafe { CStr::from_ptr(hex_ptr) }.to_string_lossy().into_owned();
        assert!(hex.len() >= 8, "{hex}");
        let out_ptr = zip_run(hex_ptr);
        zip_free(hex_ptr);
        assert!(!out_ptr.is_null());
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_string_lossy().into_owned();
        zip_free(out_ptr);
        assert!(out.contains("7"), "{out}");
    }

    #[test]
    fn ошибка_null() {
        assert!(zip_compile(std::ptr::null()).is_null());
        assert_eq!(zip_last_success(), 0);
    }
}
