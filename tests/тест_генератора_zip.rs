//! Тесты VM-генератора ZIPB (`--gen`).

use std::path::PathBuf;
use std::process::Command;

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
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

fn сгенерировать(программа: &str) -> Vec<u8> {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let файл = std::env::temp_dir().join(format!("тест_ген_{id}.zip"));
    std::fs::write(&файл, программа).unwrap();
    let out = Command::new(vm())
        .args(["--gen", файл.to_str().unwrap()])
        .current_dir(корень())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "gen fail: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

#[test]
fn тест_генератор_магия_zipb() {
    let байткод = сгенерировать("функция главная() {\n    возврат(42)\n}\n");
    assert_eq!(&байткод[0..4], b"ZIPB");
    assert_eq!(u16::from_le_bytes([байткод[4], байткод[5]]), 3);
}

#[test]
fn тест_генератор_простая_функция() {
    let байткод = сгенерировать("функция главная() {\n    возврат(42)\n}\n");
    assert!(байткод.len() > 16);
}

#[test]
fn тест_генератор_арифметика() {
    let байткод = сгенерировать("функция главная() {\n    возврат(2 + 3)\n}\n");
    assert!(байткод.len() > 16);
    assert!(байткод.contains(&0x20)); // ADD
}

#[test]
fn тест_генератор_если() {
    let байткод = сгенерировать(
        "функция главная() {\n    если (истина) { возврат(1) } иначе { возврат(0) }\n}\n",
    );
    assert!(байткод.contains(&0x52)); // JUMP_IF_NOT
}

#[test]
fn тест_генератор_поток() {
    let байткод = сгенерировать("функция главная() {\n    САПАР(1) >> ЧАТАВ!\n}\n");
    assert!(байткод.contains(&0x70)); // PULSE
}
