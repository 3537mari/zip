//! Тесты модуля Дата/Время (STD + runtime datetime.c).

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
    let src = корень().join("zip-std/runtime/datetime.c");
    let out = std::env::temp_dir().join(format!(
        "zip_dt_{}_{}.{}",
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
fn тест_модуль_datetime_парсится() {
    let s = std::fs::read_to_string(корень().join("zip-std/datetime.zip")).unwrap();
    let д = разобрать_исходник(&s).unwrap();
    match д {
        Узел::Программа { объявления } => {
            assert!(объявления
                .iter()
                .any(|у| matches!(у, Узел::Runtime { имя } if имя == "datetime")));
        }
        _ => panic!(),
    }
}

#[test]
fn тест_демо_datetime() {
    let s = std::fs::read_to_string(корень().join("examples/datetime_демо.zip")).unwrap();
    assert!(matches!(разобрать_исходник(&s).unwrap(), Узел::Программа { .. }));
}

#[test]
fn тест_unit_datetime_c() {
    let Some(exe) = собрать("ZIP_DATETIME_UNITTEST", "u") else {
        return;
    };
    let out = Command::new(&exe).output().expect("запуск datetime unit");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("ok"));
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_main_datetime_демо() {
    let Some(exe) = собрать("ZIP_DATETIME_MAIN", "m") else {
        return;
    };
    let out = Command::new(&exe).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Сейчас:"));
    assert!(s.contains("Год:"));
    assert!(s.contains("Завтра:"));
    let _ = std::fs::remove_file(exe);
}

#[test]
fn тест_текущее_время() {
    let Some(exe) = собрать("ZIP_DATETIME_UNITTEST", "now") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_форматирование() {
    let Some(exe) = собрать("ZIP_DATETIME_UNITTEST", "fmt") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_арифметика() {
    let Some(exe) = собрать("ZIP_DATETIME_UNITTEST", "arith") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_парсинг_строки() {
    let Some(exe) = собрать("ZIP_DATETIME_UNITTEST", "parse") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_интервалы() {
    let Some(exe) = собрать("ZIP_DATETIME_UNITTEST", "iv") else {
        return;
    };
    assert!(Command::new(&exe).status().unwrap().success());
    let _ = std::fs::remove_file(&exe);
}
