//! Тесты RISC-V через QEMU (пропускаются, если нет qemu / бинарника).

use std::path::Path;
use std::process::Command;

fn qemu_доступен() -> bool {
    Command::new("qemu-riscv64")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn путь_vm_riscv() -> Option<&'static str> {
    let p = "zip-vm/target/riscv64gc-unknown-linux-gnu/release/zip-vm";
    if Path::new(p).exists() {
        Some(p)
    } else {
        None
    }
}

/// RISC-V VM запускает тот же ZIP-байткод, что x86-64.
#[test]
fn тест_riscv_запуск() {
    if !qemu_доступен() {
        eprintln!("QEMU не установлен — тест пропущен");
        return;
    }
    let Some(vm) = путь_vm_riscv() else {
        eprintln!("RISC-V VM не собран — тест пропущен");
        return;
    };

    let zipb = std::env::temp_dir().join("riscv_привет.zipb");
    if !zipb.exists() {
        eprintln!("нет {} — тест пропущен (CI создаёт файл)", zipb.display());
        return;
    }

    let статус = Command::new("qemu-riscv64")
        .args(["-L", "/usr/riscv64-linux-gnu", vm])
        .arg(&zipb)
        .status()
        .expect("qemu-riscv64 не запустился");

    assert!(статус.success(), "RISC-V VM завершился с ошибкой");
}

/// selfc на RISC-V даёт идентичный ZIP-байткод.
#[test]
fn тест_riscv_fixed_point() {
    if !qemu_доступен() {
        eprintln!("QEMU не установлен — тест пропущен");
        return;
    }
    let Some(vm) = путь_vm_riscv() else {
        eprintln!("RISC-V VM не собран — тест пропущен");
        return;
    };

    let tmp = std::env::temp_dir();
    let selfc = tmp.join("selfc_v2.zipb");
    let out = tmp.join("selfc_riscv_test.zipb");
    if !selfc.exists() {
        eprintln!("нет selfc_v2.zipb — тест пропущен");
        return;
    }

    let статус = Command::new("qemu-riscv64")
        .args(["-L", "/usr/riscv64-linux-gnu", vm])
        .arg(&selfc)
        .arg("zip-self-zip/главное_чистое.zip")
        .arg(&out)
        .status()
        .expect("qemu-riscv64 не запустился");

    assert!(статус.success());

    let x86 = std::fs::read(&selfc).unwrap();
    let riscv = std::fs::read(&out).unwrap();
    assert_eq!(x86, riscv, "ZIP-байткод не платформонезависим");
}
