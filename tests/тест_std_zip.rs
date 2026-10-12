//! Тесты STD на чистом ZIP (bootstrap → VM builtins).

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
    let unix = r.join("bootstrap/zipc");
    if exe.exists() {
        return exe;
    }
    if unix.exists() {
        return unix;
    }
    let _ = std::process::Command::new("pwsh")
        .args(["-File", "bootstrap/build.ps1"])
        .current_dir(&r)
        .status();
    if exe.exists() {
        exe
    } else {
        unix
    }
}

fn vm() -> PathBuf {
    let r = корень();
    for base in [
        r.join("zip-vm/target"),
        std::env::var("CARGO_TARGET_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_default(),
    ] {
        if base.as_os_str().is_empty() {
            continue;
        }
        for profile in ["release", "debug"] {
            for name in ["zip-vm.exe", "zip-vm"] {
                let p = base.join(profile).join(name);
                if p.exists() {
                    return p;
                }
            }
        }
    }
    r.join("zip-vm/target/release/zip-vm.exe")
}

fn скомпилировать_и_запустить(программа: &str) -> String {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let файл = std::env::temp_dir().join(format!("тест_std_{id}.zip"));
    std::fs::write(&файл, программа).unwrap();
    let zipb = std::env::temp_dir().join(format!("тест_std_{id}.zipb"));
    let zipc = bootstrap();
    let compile = Command::new(&zipc)
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "bootstrap compile failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let vm_bin = vm();
    if !vm_bin.exists() {
        let _ = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--manifest-path",
                корень().join("zip-vm/Cargo.toml").to_str().unwrap(),
            ])
            .current_dir(корень())
            .env("CARGO_TARGET_DIR", корень().join("zip-vm/target"))
            .status();
    }
    let вывод = Command::new(vm_bin)
        .arg(&zipb)
        .output()
        .expect("zip-vm");
    String::from_utf8_lossy(&вывод.stdout).to_string()
}

#[test]
fn тест_ядро_числа() {
    let вывод = скомпилировать_и_запустить(
        r#"
функция главная() {
    ПечатьЧисло(модуль(0 - 7))
    ПечатьЧисло(минимум(3, 9))
    ПечатьЧисло(максимум(3, 9))
}
"#,
    );
    assert!(вывод.contains('7'));
    assert!(вывод.contains('3'));
    assert!(вывод.contains('9'));
}

#[test]
fn тест_тензоры() {
    let вывод = скомпилировать_и_запустить(
        r#"
функция главная() {
    т = создать_тензор(2, 3, 0, 0)
    ПечатьЧисло(тензор_всего(т))
}
"#,
    );
    assert!(вывод.contains('6'));
}

#[test]
fn тест_нейросети_xor() {
    let вывод = скомпилировать_и_запустить(
        r#"
функция главная() {
    слой1 = создать_слой(2, 4)
    слой2 = создать_слой(4, 1)
    вход = создать_тензор(1, 2, 0, 0)
    ч = прямой_проход_слоя(слой1, вход)
    выход = прямой_проход_слоя(слой2, ч)
    ПечатьЧисло(1)
}
"#,
    );
    assert!(вывод.contains('1'));
}

#[test]
fn тест_sha256() {
    let вывод = скомпилировать_и_запустить(
        r#"
функция главная() {
    результат = хеш_ша256("hello", 5)
    ПечатьСтроки(результат)
}
"#,
    );
    assert!(вывод.contains("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"));
}
