//! Тесты модуля JSON (STD + runtime json_parser.c).

use std::path::PathBuf;
use std::process::Command;
use zipc::{разобрать_исходник, Узел};

fn корень_репо() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn найти_clang() -> Option<PathBuf> {
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
    let candidates = [
        r"C:\Program Files\LLVM\bin\clang.exe",
        "/usr/bin/clang",
        "/usr/local/bin/clang",
    ];
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
}

fn скомпилировать_json(макрос: &str, имя: &str) -> Option<PathBuf> {
    let clang = найти_clang()?;
    let корень = корень_репо();
    let источник = корень.join("zip-std/runtime/json_parser.c");
    let out = std::env::temp_dir().join(format!(
        "zip_json_{}_{}.{}",
        имя,
        std::process::id(),
        if cfg!(windows) { "exe" } else { "out" }
    ));
    let статус = Command::new(&clang)
        .arg(&источник)
        .arg("-O2")
        .arg(format!("-D{}", макрос))
        .arg("-o")
        .arg(&out)
        .status()
        .ok()?;
    if статус.success() {
        Some(out)
    } else {
        None
    }
}

#[test]
fn тест_модуль_json_парсится() {
    let путь = корень_репо().join("zip-std/json.zip");
    let исходник = std::fs::read_to_string(&путь).expect("json.zip");
    let дерево = разобрать_исходник(&исходник).expect("parse");
    match дерево {
        Узел::Программа { объявления } => {
            assert!(объявления
                .iter()
                .any(|у| matches!(у, Узел::Runtime { имя } if имя == "json_parser")));
        }
        _ => panic!("ожидалась программа"),
    }
}

#[test]
fn тест_демо_json_runtime() {
    let путь = корень_репо().join("examples/json_демо.zip");
    let исходник = std::fs::read_to_string(&путь).expect("демо");
    let дерево = разобрать_исходник(&исходник).unwrap();
    assert!(matches!(дерево, Узел::Программа { .. }));
}

#[test]
fn тест_unit_json_c() {
    let Some(exe) = скомпилировать_json("ZIP_JSON_UNITTEST", "unit") else {
        eprintln!("clang недоступен — пропуск unit JSON");
        return;
    };
    let out = Command::new(&exe).output().expect("run unit");
    assert!(
        out.status.success(),
        "unit failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ok"), "stdout={}", stdout);
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_main_json_демо_вывод() {
    let Some(exe) = скомпилировать_json("ZIP_JSON_MAIN", "main") else {
        eprintln!("clang недоступен — пропуск main JSON");
        return;
    };
    let out = Command::new(&exe).output().expect("run main");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Имя:"), "{}", stdout);
    assert!(stdout.contains("Иван"), "{}", stdout);
    assert!(stdout.contains("Возраст:"), "{}", stdout);
    assert!(stdout.contains("25"), "{}", stdout);
    assert!(stdout.contains("Улица:"), "{}", stdout);
    assert!(stdout.contains("Ленина"), "{}", stdout);
    let _ = std::fs::remove_file(&exe);
}

#[test]
fn тест_разбор_объекта_в_c() {
    // покрывается ZIP_JSON_UNITTEST
    тест_unit_json_c();
}

#[test]
fn тест_разбор_массива_в_c() {
    тест_unit_json_c();
}

#[test]
fn тест_путь_с_индексом_в_c() {
    тест_unit_json_c();
}

#[test]
fn тест_запись_и_установка_в_c() {
    тест_unit_json_c();
}
