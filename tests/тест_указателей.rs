//! v6: указатели и память в C-bootstrap.

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

fn скомпилировать(программа: &str) -> bool {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let файл = std::env::temp_dir().join(format!("тест_ук_{}.zip", id));
    let zipb = std::env::temp_dir().join(format!("тест_ук_{}.zipb", id));

    std::fs::write(&файл, программа).unwrap();

    Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .status()
        .map(|с| с.success())
        .unwrap_or(false)
}

#[test]
fn тест_взять_адрес() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            х = 42
            п = &х
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_разыменовать() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            х = 42
            п = &х
            значение = *п
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_записать_по_указателю() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            х = 42
            п = &х
            *п = 100
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_выделить_память() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            буфер = выделить(100)
            *буфер = 42
            освободить(буфер)
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_арифметика_указателей() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            п = выделить(100)
            п2 = п + 10
            п3 = п2 - 5
            освободить(п)
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_динамический_массив() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            n = 100
            числа = выделить(n * 8)

            для и в 0..n {
                *(числа + и * 8) = и
            }

            освободить(числа)
            возврат(0)
        }
    "#
    ));
}
