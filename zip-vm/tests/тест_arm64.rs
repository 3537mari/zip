//! Тесты ARM64 через QEMU (пропускаются, если нет qemu / бинарника).

use std::path::Path;
use std::process::Command;

fn qemu_доступен() -> bool {
    Command::new("qemu-aarch64")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn путь_vm_arm() -> Option<&'static str> {
    let p = "zip-vm/target/aarch64-unknown-linux-gnu/release/zip-vm";
    if Path::new(p).exists() {
        Some(p)
    } else {
        None
    }
}

/// ARM64 VM запускает тот же ZIP-байткод, что x86-64.
#[test]
fn тест_arm64_запуск() {
    if !qemu_доступен() {
        eprintln!("QEMU не установлен — тест пропущен");
        return;
    }
    let Some(vm_arm) = путь_vm_arm() else {
        eprintln!("ARM64 VM не собран — тест пропущен");
        return;
    };

    let zipb = std::env::temp_dir().join("arm64_hello.zipb");
    if !zipb.exists() {
        eprintln!("нет {} — тест пропущен (CI создаёт файл)", zipb.display());
        return;
    }

    let статус = Command::new("qemu-aarch64")
        .args(["-L", "/usr/aarch64-linux-gnu", vm_arm])
        .arg(&zipb)
        .status()
        .expect("qemu-aarch64 не запустился");

    assert!(статус.success(), "ARM64 VM завершился с ошибкой");
}

/// selfc на ARM64 даёт идентичный ZIP-байткод.
#[test]
fn тест_arm64_fixed_point() {
    if !qemu_доступен() {
        eprintln!("QEMU не установлен — тест пропущен");
        return;
    }
    let Some(vm_arm) = путь_vm_arm() else {
        eprintln!("ARM64 VM не собран — тест пропущен");
        return;
    };

    let tmp = std::env::temp_dir();
    let selfc = tmp.join("selfc_v2.zipb");
    let out = tmp.join("selfc_arm_test.zipb");
    if !selfc.exists() {
        eprintln!("нет selfc_v2.zipb — тест пропущен");
        return;
    }

    let статус = Command::new("qemu-aarch64")
        .args(["-L", "/usr/aarch64-linux-gnu", vm_arm])
        .arg(&selfc)
        .arg("zip-self-zip/главное_чистое.zip")
        .arg(&out)
        .status()
        .expect("qemu-aarch64 не запустился");

    assert!(статус.success());

    let x86 = std::fs::read(&selfc).unwrap();
    let arm = std::fs::read(&out).unwrap();
    assert_eq!(x86, arm, "ZIP-байткод не платформонезависим");
}
