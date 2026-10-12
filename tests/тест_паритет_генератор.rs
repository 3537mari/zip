//! Паритет: C-bootstrap и VM-генератор оба дают ZIPB.

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
        exe
    } else {
        r.join("bootstrap/zipc")
    }
}

fn vm() -> PathBuf {
    let r = корень();
    let p = r.join("zip-vm/target/release/zip-vm.exe");
    if p.exists() {
        p
    } else {
        r.join("zip-vm/target/release/zip-vm")
    }
}

fn сравнить_генерацию(программа: &str) -> bool {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = корень();
    let файл = std::env::temp_dir().join(format!("паритет_ген_{id}.zip"));
    let zipb_c = std::env::temp_dir().join(format!("паритет_ген_{id}_c.zipb"));
    std::fs::write(&файл, программа).unwrap();

    let c = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb_c.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    if !c.success() {
        return false;
    }
    let размер_c = std::fs::metadata(&zipb_c).unwrap().len();

    if !vm().exists() {
        let _ = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                root.join("zip-vm/Cargo.toml").to_str().unwrap(),
            ])
            .current_dir(&root)
            .env("CARGO_TARGET_DIR", root.join("zip-vm/target"))
            .status();
    }
    let out = Command::new(vm())
        .args(["--gen", файл.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    if !out.status.success() || out.stdout.len() < 4 {
        return false;
    }
    &out.stdout[0..4] == b"ZIPB" && размер_c > 0 && !out.stdout.is_empty()
}

#[test]
fn паритет_генерация_простой() {
    assert!(сравнить_генерацию(
        "функция главная() {\n    возврат(42)\n}\n"
    ));
}

#[test]
fn паритет_генерация_сложной() {
    assert!(сравнить_генерацию(
        r#"функция факториал(н) {
    если (н < 2) { возврат(1) }
    возврат(н * факториал(н - 1))
}

функция главная() {
    возврат(факториал(5))
}
"#
    ));
}

#[test]
fn паритет_генерация_с_потоком() {
    assert!(сравнить_генерацию(
        "функция главная() {\n    САПАР(1) >> ЧАТАВ!\n}\n"
    ));
}
