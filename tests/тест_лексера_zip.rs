//! Тесты полного лексера (алгоритм без C-bootstrap).

use std::path::PathBuf;
use std::process::Command;

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn bootstrap() -> PathBuf {
    let r = корень();
    let exe = r.join("bootstrap/zipc.exe");
    if exe.exists() {
        return exe;
    }
    let unix = r.join("bootstrap/zipc");
    if !unix.exists() {
        let _ = Command::new("pwsh")
            .args(["-File", "bootstrap/build.ps1"])
            .current_dir(&r)
            .status();
    }
    if exe.exists() {
        exe
    } else {
        unix
    }
}

fn vm() -> PathBuf {
    let r = корень();
    let p = r.join("zip-vm/target/release/zip-vm.exe");
    if p.exists() {
        return p;
    }
    let _ = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--manifest-path",
            r.join("zip-vm/Cargo.toml").to_str().unwrap(),
        ])
        .current_dir(&r)
        .env("CARGO_TARGET_DIR", r.join("zip-vm/target"))
        .status();
    if p.exists() {
        p
    } else {
        r.join("zip-vm/target/release/zip-vm")
    }
}

fn escape_zip_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn lex_count(исходник: &str) -> String {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let файл = std::env::temp_dir().join(format!("тест_лексер_{id}.zip"));
    let zipb = std::env::temp_dir().join(format!("тест_лексер_{id}.zipb"));
    let body = format!(
        "функция главная() {{\n    н = zip_lex_полный(\"{}\")\n    ПечатьЧисло(н)\n}}\n",
        escape_zip_str(исходник)
    );
    std::fs::write(&файл, body).unwrap();
    let compile = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "compile: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let out = Command::new(vm())
        .arg(&zipb)
        .current_dir(корень())
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn тест_лексер_пустой_файл() {
    let вывод = lex_count("");
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_лексер_число() {
    let вывод = lex_count("42");
    assert!(вывод.contains('2'), "got: {вывод}");
}

#[test]
fn тест_лексер_кириллица() {
    let вывод = lex_count("функция");
    assert!(вывод.contains('2'), "got: {вывод}");
}

#[test]
fn тест_лексер_корень() {
    let вывод = lex_count("СПР");
    assert!(вывод.contains('2'), "got: {вывод}");
}

#[test]
fn тест_лексер_шаблон() {
    let вывод = lex_count("САПАР");
    assert!(вывод.contains('2'), "got: {вывод}");
}

#[test]
fn тест_лексер_операторы() {
    let вывод = lex_count(">> >>> || ! ~");
    assert!(вывод.contains('6'), "got: {вывод}");
}

#[test]
fn тест_лексер_паритет_с_полным_модулем() {
    let root = корень();
    let src = root.join("zip-self-zip/лексика_полная.zip");
    assert!(src.exists());
    let zipb = std::env::temp_dir().join("лексика_полная_test.zipb");
    let c = Command::new(bootstrap())
        .args([src.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
    let out = Command::new(vm())
        .arg(&zipb)
        .current_dir(&root)
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('1'), "{s}");
    assert!(s.contains('2'), "{s}");
    assert!(s.contains('6'), "{s}");
}
