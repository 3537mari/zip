//! Параллельный JIT — компиляция горячих функций в нескольких потоках.
//!
//! Без tokio: `std::thread` + каналы. Каждый поток пишет свой `.ll`/DLL
//! и возвращает `(id, путь, символ)`; загрузка `Library` — в вызывающем потоке
//! (чтобы DLL не выгрузились).

use crate::байткод::Функция;
use crate::байткод_в_llvm::{байткод_в_llvm_ir, имя_символа};
use crate::jit::JitFn;
use libloading::Library;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;

static СЧЁТЧИК_DLL: AtomicU64 = AtomicU64::new(1);

/// Планировщик параллельной JIT-компиляции.
pub struct ПараллельныйJIT {
    количество_потоков: usize,
    /// Держим DLL, пока жив JIT.
    библиотеки: Vec<Library>,
    готовые: HashMap<u32, JitFn>,
}

impl ПараллельныйJIT {
    pub fn новый(количество_потоков: usize) -> Self {
        Self {
            количество_потоков: количество_потоков.max(1),
            библиотеки: Vec::new(),
            готовые: HashMap::new(),
        }
    }

    pub fn из_окружения() -> Self {
        let n = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2)
            .min(8);
        Self::новый(n)
    }

    pub fn получить(&self, ид: u32) -> Option<JitFn> {
        self.готовые.get(&ид).copied()
    }

    pub fn готовые(&self) -> &HashMap<u32, JitFn> {
        &self.готовые
    }

    /// Скомпилировать пакет функций параллельно.
    pub fn скомпилировать_пакет(&mut self, задачи: Vec<(u32, Функция)>) -> HashMap<u32, Result<JitFn, String>> {
        let mut итог: HashMap<u32, Result<JitFn, String>> = HashMap::new();
        if задачи.is_empty() {
            return итог;
        }

        // Уже готовые — сразу
        let mut остаток = Vec::new();
        for (ид, ф) in задачи {
            if let Some(f) = self.готовые.get(&ид) {
                итог.insert(ид, Ok(*f));
            } else {
                остаток.push((ид, ф));
            }
        }
        if остаток.is_empty() {
            return итог;
        }

        let clang = match найти_clang() {
            Some(c) => c,
            None => {
                for (ид, _) in остаток {
                    итог.insert(ид, Err("clang не найден".into()));
                }
                return итог;
            }
        };

        let потоки = self.количество_потоков.min(остаток.len()).max(1);
        let размер_батча = (остаток.len() + потоки - 1) / потоки;
        let (tx, rx) = mpsc::channel::<(u32, Result<PathBuf, String>)>();

        let mut handles = Vec::new();
        for батч in остаток.chunks(размер_батча) {
            let батч = батч.to_vec();
            let clang = clang.clone();
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                for (ид, функция) in батч {
                    let r = скомпилировать_в_dll(&clang, ид, &функция);
                    let _ = tx.send((ид, r));
                }
            }));
        }
        drop(tx);

        for (ид, путь_рез) in rx {
            match путь_рез {
                Ok(путь) => match загрузить_символ(&путь, &имя_символа(ид), &mut self.библиотеки) {
                    Ok(f) => {
                        self.готовые.insert(ид, f);
                        итог.insert(ид, Ok(f));
                        eprintln!("[JIT-параллель] функция {} → {:?}", ид, f as *const ());
                    }
                    Err(е) => {
                        итог.insert(ид, Err(е));
                    }
                },
                Err(е) => {
                    итог.insert(ид, Err(е));
                }
            }
        }

        for h in handles {
            let _ = h.join();
        }

        итог
    }
}

fn скомпилировать_в_dll(clang: &Path, id: u32, функция: &Функция) -> Result<PathBuf, String> {
    let ir = байткод_в_llvm_ir(функция, id)?;
    let рабочая = std::env::temp_dir().join("zip-jit-parallel");
    let _ = std::fs::create_dir_all(&рабочая);
    let уникальный = format!(
        "p{}_{}_{}",
        id,
        std::process::id(),
        СЧЁТЧИК_DLL.fetch_add(1, Ordering::Relaxed)
    );
    let путь_ll = рабочая.join(format!("{}.ll", уникальный));
    std::fs::write(&путь_ll, &ir).map_err(|е| format!("запись .ll: {}", е))?;

    let путь_lib = if cfg!(windows) {
        рабочая.join(format!("{}.dll", уникальный))
    } else {
        рабочая.join(format!("lib{}.so", уникальный))
    };

    let mut cmd = std::process::Command::new(clang);
    cmd.arg("-O3").arg("-march=native").arg("-shared");
    if cfg!(target_arch = "x86_64") {
        cmd.arg("-mavx2");
    }
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
    Ok(путь_lib)
}

fn найти_clang() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ZIP_CLANG") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    which("clang").or_else(|| {
        [
            r"C:\Program Files\LLVM\bin\clang.exe",
            "/usr/bin/clang",
            "/usr/local/bin/clang",
        ]
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

#[cfg(test)]
mod тесты {
    use super::*;
    use crate::байткод::Инструкция;

    #[test]
    fn пакет_пустой() {
        let mut jit = ПараллельныйJIT::новый(2);
        let r = jit.скомпилировать_пакет(vec![]);
        assert!(r.is_empty());
    }

    #[test]
    fn пакет_простых_функций() {
        if std::env::var_os("ZIP_JIT").as_deref() == Some(std::ffi::OsStr::new("0")) {
            return;
        }
        if найти_clang().is_none() {
            return;
        }
        let ф1 = Функция {
            имя: "a".into(),
            параметры: 1,
            локальные: 2,
            размер_стека: 8,
            порог_горячности: 1,
            код: vec![Инструкция::Load(0), Инструкция::Ret],
        };
        let ф2 = Функция {
            имя: "b".into(),
            параметры: 1,
            локальные: 2,
            размер_стека: 8,
            порог_горячности: 1,
            код: vec![
                Инструкция::Load(0),
                Инструкция::PushI64(1),
                Инструкция::Add,
                Инструкция::Ret,
            ],
        };
        let mut jit = ПараллельныйJIT::новый(2);
        let r = jit.скомпилировать_пакет(vec![(10, ф1), (11, ф2)]);
        assert!(r.get(&10).map(|x| x.is_ok()).unwrap_or(false));
        assert!(r.get(&11).map(|x| x.is_ok()).unwrap_or(false));
    }
}
