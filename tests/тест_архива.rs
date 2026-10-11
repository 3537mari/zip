//! Тесты модуля ZIP-архивов (STD + runtime zip_archive.c).

use std::path::PathBuf;
use std::process::Command;
use zipc::{разобрать_исходник, Узел};

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn clang() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ZIP_CLANG") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for name in ["clang", "clang.exe"] {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    let c = PathBuf::from(r"C:\Program Files\LLVM\bin\clang.exe");
    c.is_file().then_some(c)
}

fn собрать(макрос: &str, имя: &str) -> Option<PathBuf> {
    let clang = clang()?;
    let src = корень().join("zip-std/runtime/zip_archive.c");
    let out = std::env::temp_dir().join(format!(
        "zip_arch_{}_{}.{}",
        имя,
        std::process::id(),
        if cfg!(windows) { "exe" } else { "out" }
    ));
    let ok = Command::new(clang)
        .arg(&src)
        .arg("-O2")
        .arg(format!("-D{}", макрос))
        .arg("-o")
        .arg(&out)
        .status()
        .ok()?
        .success();
    ok.then_some(out)
}

#[test]
fn тест_модуль_архив_парсится() {
    let s = std::fs::read_to_string(корень().join("zip-std/архив.zip")).unwrap();
    let д = разобрать_исходник(&s).unwrap();
    match д {
        Узел::Программа { объявления } => {
            assert!(объявления
                .iter()
                .any(|у| matches!(у, Узел::Runtime { имя } if имя == "zip_archive")));
        }
        _ => panic!(),
    }
}

#[test]
fn тест_демо_архив() {
    let s = std::fs::read_to_string(корень().join("examples/архив_демо.zip")).unwrap();
    assert!(matches!(разобрать_исходник(&s).unwrap(), Узел::Программа { .. }));
}

#[test]
fn тест_unit_архив_c() {
    let Some(exe) = собрать("ZIP_ARCHIVE_UNITTEST", "u") else {
        return;
    };
    let out = Command::new(&exe).output().expect("запуск archive unit");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("ok"));
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_main_архив_демо() {
    let Some(exe) = собрать("ZIP_ARCHIVE_MAIN", "m") else {
        return;
    };
    let out = Command::new(&exe).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Файлов в архиве:"));
    assert!(s.contains("привет.txt"));
    assert!(s.contains("Содержимое:"));
    let _ = std::fs::remove_file(exe);
}

#[test]
fn тест_открыть_архив() {
    let Some(exe) = собрать("ZIP_ARCHIVE_UNITTEST", "open") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_извлечь_файл() {
    let Some(exe) = собрать("ZIP_ARCHIVE_UNITTEST", "extract") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_отсутствующий_файл() {
    let Some(exe) = собрать("ZIP_ARCHIVE_UNITTEST", "miss") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}
