//! Тесты Windows ARM64 (пропускаются, если нет бинарника / хост не ARM64).

use std::path::Path;
use std::process::Command;

fn путь_vm_win_arm() -> Option<&'static str> {
    let p = "zip-vm/target/aarch64-pc-windows-msvc/release/zip-vm.exe";
    if Path::new(p).exists() {
        Some(p)
    } else {
        None
    }
}

/// Бинарник Windows ARM собран и имеет разумный размер.
#[test]
fn тест_windows_arm_бинарник_собран() {
    let Some(vm) = путь_vm_win_arm() else {
        eprintln!("Windows ARM VM не собран — тест пропущен");
        return;
    };
    let размер = std::fs::metadata(vm).unwrap().len();
    assert!(размер > 100_000, "Подозрительно маленький бинарник: {размер}");
    assert!(размер < 20_000_000, "Подозрительно большой бинарник: {размер}");
}

/// На хосте ARM64 — нативный запуск zip-vm.
#[test]
#[cfg(target_os = "windows")]
fn тест_windows_arm_запуск() {
    let Some(vm) = путь_vm_win_arm() else {
        eprintln!("Windows ARM VM не собран — тест пропущен");
        return;
    };

    let arch = std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_default();
    if arch != "ARM64" {
        eprintln!("Хост — не ARM64 ({arch}), запуск пропущен");
        return;
    }

    let zipb = std::env::temp_dir().join("zip-winarm").join("win_arm_test.zipb");
    if !zipb.exists() {
        eprintln!("нет {} — тест пропущен (CI создаёт файл)", zipb.display());
        return;
    }

    let статус = Command::new(vm)
        .arg(&zipb)
        .status()
        .expect("VM не запустился");
    assert!(статус.success(), "Windows ARM VM завершился с ошибкой");
}
