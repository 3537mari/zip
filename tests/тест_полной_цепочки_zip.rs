//! Полная цепочка: лексер → парсер → генератор → ZIPB (в VM).

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

fn проверить_цепочку(программа: &str) -> bool {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = корень();
    let файл = std::env::temp_dir().join(format!("цепочка_{id}.zip"));
    let zipb_c = std::env::temp_dir().join(format!("цепочка_{id}_c.zipb"));
    std::fs::write(&файл, программа).unwrap();

    let статус_c = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb_c.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    if !статус_c.success() {
        return false;
    }

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

    let вывод_vm = Command::new(vm())
        .args(["--gen", файл.to_str().unwrap()])
        .current_dir(&root)
        .output()
        .unwrap();
    вывод_vm.status.success()
        && вывод_vm.stdout.len() >= 4
        && &вывод_vm.stdout[0..4] == b"ZIPB"
}

#[test]
fn цепочка_простая_программа() {
    assert!(проверить_цепочку(
        "функция главная() {\n    возврат(42)\n}\n"
    ));
}

#[test]
fn цепочка_с_арифметикой() {
    assert!(проверить_цепочку(
        "функция главная() {\n    результат = (2 + 3) * (4 - 1)\n    возврат(результат)\n}\n"
    ));
}

#[test]
fn цепочка_с_условием() {
    assert!(проверить_цепочку(
        "функция главная() {\n    если (истина) {\n        возврат(1)\n    } иначе {\n        возврат(0)\n    }\n}\n"
    ));
}

#[test]
fn цепочка_с_циклом() {
    assert!(проверить_цепочку(
        "функция главная() {\n    сумма = 0\n    для и в 0..10 {\n        сумма = сумма + и\n    }\n    возврат(сумма)\n}\n"
    ));
}

#[test]
fn цепочка_с_рекурсией() {
    assert!(проверить_цепочку(
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
fn цепочка_с_потоком() {
    assert!(проверить_цепочку(
        "функция главная() {\n    САПАР(1) >> ЧАТАВ!\n}\n"
    ));
}

#[test]
fn цепочка_с_аннотациями() {
    assert!(проверить_цепочку(
        "@ускорить(уровень = 3)\n@кэшировать(размер = 1024)\nфункция быстрая(х) {\n    возврат(х * 2)\n}\n"
    ));
}
