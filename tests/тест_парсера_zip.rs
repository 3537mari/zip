//! Тесты полного парсера (VM, без C-bootstrap).

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

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn parse_ok(исходник: &str) -> String {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let файл = std::env::temp_dir().join(format!("тест_парсер_{id}.zip"));
    let zipb = std::env::temp_dir().join(format!("тест_парсер_{id}.zipb"));
    let body = format!(
        "функция главная() {{\n    р = zip_self_parse(\"{}\")\n    ПечатьЧисло(р)\n}}\n",
        escape(исходник)
    );
    std::fs::write(&файл, body).unwrap();
    let c = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .output()
        .unwrap();
    assert!(
        c.status.success(),
        "compile: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let out = Command::new(vm())
        .arg(&zipb)
        .current_dir(корень())
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn тест_парсер_пустая_программа() {
    let вывод = parse_ok("");
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_функция() {
    let вывод = parse_ok("функция главная() { возврат(42) }");
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_если() {
    let вывод = parse_ok(
        "функция главная() { если (истина) { возврат(1) } иначе { возврат(0) } }",
    );
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_поток() {
    let вывод = parse_ok("функция главная() { САПАР(1) >> ЧАТАВ! }");
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_приоритеты() {
    let вывод = parse_ok("функция главная() { результат = 2 + 3 * 4\nвозврат(результат) }");
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_аннотации() {
    let вывод = parse_ok(
        "@ускорить(уровень = 3)\n@кэшировать(размер = 1024)\nфункция быстрая(х) { возврат(х * 2) }",
    );
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_циклы() {
    let вывод = parse_ok(
        "функция главная() { сумма = 0\nдля и в 0..10 { сумма = сумма + и }\nвозврат(сумма) }",
    );
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_пока() {
    let вывод = parse_ok(
        "функция главная() { х = 0\nпока (х < 10) { х = х + 1 }\nвозврат(х) }",
    );
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_параметры_с_типами() {
    let вывод = parse_ok(
        "функция сложить(а: Целое, б: Целое) -> Целое { возврат(а + б) }",
    );
    assert!(вывод.contains('1'), "got: {вывод}");
}

#[test]
fn тест_парсер_сложное_выражение() {
    let вывод = parse_ok(
        "функция главная() { результат = (2 + 3) * (4 - 1) / 2\nвозврат(результат) }",
    );
    assert!(вывод.contains('1'), "got: {вывод}");
}
