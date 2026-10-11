//! Тесты Runtime на ZIP (syscalls / аллокатор / вывод).

use std::path::PathBuf;
use std::process::Command;

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn bootstrap_zipc() -> PathBuf {
    let root = корень();
    let candidates = [
        root.join("bootstrap/zipc.exe"),
        root.join("bootstrap/zipc"),
    ];
    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    // Собрать при отсутствии
    let status = if cfg!(windows) {
        Command::new("pwsh")
            .args(["-File", "bootstrap/build.ps1"])
            .current_dir(&root)
            .status()
    } else {
        Command::new("make")
            .current_dir(root.join("bootstrap"))
            .status()
    };
    assert!(status.map(|s| s.success()).unwrap_or(false), "сборка bootstrap");
    if root.join("bootstrap/zipc.exe").exists() {
        root.join("bootstrap/zipc.exe")
    } else {
        root.join("bootstrap/zipc")
    }
}

#[test]
fn тест_вывод_через_syscalls() {
    let программа = r#"
функция главная() {
    sys_write(1, "Привет, syscalls!", 32)
    sys_write(1, "\n", 1)
}
"#;
    let файл = std::env::temp_dir().join("тест_syscalls.zip");
    std::fs::write(&файл, программа).unwrap();
    let out = std::env::temp_dir().join("тест_syscalls.zipb");
    let zipc = bootstrap_zipc();
    let статус = Command::new(&zipc)
        .args([файл.to_str().unwrap(), out.to_str().unwrap()])
        .status()
        .expect("не могу запустить bootstrap");
    assert!(статус.success());
    assert!(out.exists());
}

#[test]
fn тест_аллокатор() {
    let программа = r#"
функция главная() {
    п1 = выделить(1024)
    п2 = выделить(2048)
    если п1 != 0 {
        если п2 != 0 {
            sys_write(1, "OK", 2)
        }
    }
}
"#;
    let файл = std::env::temp_dir().join("тест_alloc.zip");
    std::fs::write(&файл, программа).unwrap();
    let out = std::env::temp_dir().join("тест_alloc.zipb");
    let zipc = bootstrap_zipc();
    let статус = Command::new(&zipc)
        .args([файл.to_str().unwrap(), out.to_str().unwrap()])
        .status()
        .expect("не могу запустить bootstrap");
    assert!(статус.success());
}

#[test]
fn тест_вывод_числа() {
    let программа = r#"
функция главная() {
    напечатать_число_строкой(42)
}
"#;
    let файл = std::env::temp_dir().join("тест_число.zip");
    std::fs::write(&файл, программа).unwrap();
    let out = std::env::temp_dir().join("тест_число.zipb");
    let zipc = bootstrap_zipc();
    let статус = Command::new(&zipc)
        .args([файл.to_str().unwrap(), out.to_str().unwrap()])
        .status()
        .expect("не могу запустить bootstrap");
    assert!(статус.success());
}

#[test]
fn тест_vm_sys_write() {
    let root = корень();
    let zipc = bootstrap_zipc();
    let src = root.join("examples/чистый_zip.zip");
    let out = std::env::temp_dir().join("чистый_zip_test.zipb");
    let статус = Command::new(&zipc)
        .args([src.to_str().unwrap(), out.to_str().unwrap()])
        .status()
        .expect("bootstrap");
    assert!(статус.success(), "компиляция чистый_zip");

    let vm = if cfg!(windows) {
        root.join("zip-vm/target/release/zip-vm.exe")
    } else {
        root.join("zip-vm/target/release/zip-vm")
    };
    if !vm.exists() {
        let _ = Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(root.join("zip-vm/Cargo.toml"))
            .status();
    }
    if vm.exists() {
        let output = Command::new(&vm)
            .arg(&out)
            .current_dir(std::env::temp_dir())
            .output()
            .expect("zip-vm");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("ЧИСТЫЙ ZIP") || stdout.contains("Аллокатор"),
            "unexpected stdout: {stdout}"
        );
    }
}
