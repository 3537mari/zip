//! Паритет: C-bootstrap компилирует ↔ VM-парсер разбирает.

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

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn сравнить_парсинг(программа: &str) -> bool {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = корень();
    let файл = std::env::temp_dir().join(format!("паритет_{id}.zip"));
    let zipb = std::env::temp_dir().join(format!("паритет_{id}.zipb"));
    std::fs::write(&файл, программа).unwrap();

    // 1. C-bootstrap должен скомпилировать
    let c = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    if !c.success() {
        return false;
    }

    // 2. VM-парсер должен принять тот же исходник
    let probe = std::env::temp_dir().join(format!("паритет_probe_{id}.zip"));
    let probe_b = std::env::temp_dir().join(format!("паритет_probe_{id}.zipb"));
    let body = format!(
        "функция главная() {{\n    ПечатьЧисло(zip_self_parse(\"{}\"))\n}}\n",
        escape(программа)
    );
    std::fs::write(&probe, body).unwrap();
    let c2 = Command::new(bootstrap())
        .args([probe.to_str().unwrap(), probe_b.to_str().unwrap()])
        .current_dir(&root)
        .status()
        .unwrap();
    if !c2.success() {
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
    let out = Command::new(vm())
        .arg(&probe_b)
        .current_dir(&root)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains('1')
}

#[test]
fn паритет_простой_функции() {
    assert!(сравнить_парсинг(
        "функция главная() {\n    возврат(42)\n}\n"
    ));
}

#[test]
fn паритет_сложной_программы() {
    assert!(сравнить_парсинг(
        r#"@ускорить(уровень = 3)
функция факториал(н) {
    если (н < 2) { возврат(1) }
    возврат(н * факториал(н - 1))
}

функция главная() {
    результат = факториал(10)
    возврат(результат)
}
"#
    ));
}
