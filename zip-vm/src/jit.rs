//! JIT-компилятор горячих функций (Идея 5 — Миелинизация).
//!
//! Подход B: `.ll` → `clang -shared` → DLL/SO → `libloading` → нативный вызов.
//! `llvm-config` на машине нет — ORC через llvm-sys не используем.

use crate::байткод::Функция;
use crate::байткод_в_llvm::{байткод_в_llvm_ir, имя_символа};
use libloading::Library;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Сигнатура всех JIT-функций: до 4 аргументов i64 → i64.
pub type JitFn = unsafe extern "C" fn(i64, i64, i64, i64) -> i64;

pub struct JitКомпилятор {
    рабочая_директория: PathBuf,
    /// Держим Library, чтобы DLL не выгрузилась.
    #[allow(dead_code)]
    библиотеки: Vec<Library>,
    функции: HashMap<u32, JitFn>,
}

impl JitКомпилятор {
    pub fn новый() -> Self {
        let рабочая = std::env::temp_dir().join("zip-jit");
        let _ = std::fs::create_dir_all(&рабочая);
        Self {
            рабочая_директория: рабочая,
            библиотеки: Vec::new(),
            функции: HashMap::new(),
        }
    }

    pub fn доступен() -> bool {
        if std::env::var_os("ZIP_JIT").as_deref() == Some(std::ffi::OsStr::new("0")) {
            return false;
        }
        найти_clang().is_some()
    }

    pub fn получить(&self, id: u32) -> Option<JitFn> {
        self.функции.get(&id).copied()
    }

    /// Скомпилировать функцию байт-кода в нативный код.
    pub fn скомпилировать_функцию(
        &mut self,
        id: u32,
        функция: &Функция,
    ) -> Result<JitFn, String> {
        if let Some(f) = self.функции.get(&id) {
            return Ok(*f);
        }

        let clang = найти_clang().ok_or_else(|| "clang не найден в PATH".to_string())?;
        let ir = байткод_в_llvm_ir(функция, id)?;
        // Уникальное имя: иначе параллельные тесты / повторные JIT делят одну DLL.
        let уникальный = format!(
            "fn{}_{}_{}",
            id,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let путь_ll = self.рабочая_директория.join(format!("{}.ll", уникальный));
        std::fs::write(&путь_ll, &ir).map_err(|е| format!("запись .ll: {}", е))?;

        let путь_lib = if cfg!(windows) {
            self.рабочая_директория.join(format!("{}.dll", уникальный))
        } else {
            self.рабочая_директория
                .join(format!("lib{}.so", уникальный))
        };

        let mut cmd = Command::new(&clang);
        cmd.arg("-O3").arg("-march=native").arg("-shared");
        if !cfg!(windows) {
            cmd.arg("-fPIC");
        }
        cmd.arg(&путь_ll).arg("-o").arg(&путь_lib);

        let вывод = cmd
            .output()
            .map_err(|е| format!("запуск clang: {}", е))?;
        if !вывод.status.success() {
            let stderr = String::from_utf8_lossy(&вывод.stderr);
            return Err(format!("clang ошибка: {}", stderr.trim()));
        }

        let f = загрузить_символ(&путь_lib, &имя_символа(id), &mut self.библиотеки)?;
        self.функции.insert(id, f);
        eprintln!(
            "[JIT] Функция {} («{}») скомпилирована → {} @{:?}",
            id,
            функция.имя,
            путь_lib.display(),
            f as *const ()
        );
        Ok(f)
    }
}

fn найти_clang() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ZIP_CLANG") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    which("clang").or_else(|| {
        let candidates = [
            r"C:\Program Files\LLVM\bin\clang.exe",
            "/usr/bin/clang",
            "/usr/local/bin/clang",
        ];
        candidates
            .iter()
            .map(PathBuf::from)
            .find(|p| p.is_file())
    })
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
        let p_exe = dir.join(format!("{}.exe", name));
        if p_exe.is_file() {
            return Some(p_exe);
        }
    }
    None
}

fn загрузить_символ(
    путь: &Path,
    символ: &str,
    библиотеки: &mut Vec<Library>,
) -> Result<JitFn, String> {
    unsafe {
        let lib = Library::new(путь).map_err(|е| format!("LoadLibrary {:?}: {}", путь, е))?;
        let sym: libloading::Symbol<JitFn> = lib
            .get(символ.as_bytes())
            .map_err(|е| format!("GetProcAddress {}: {}", символ, е))?;
        let f: JitFn = *sym;
        библиотеки.push(lib);
        Ok(f)
    }
}
