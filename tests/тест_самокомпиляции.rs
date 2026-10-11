//! Самокомпиляция: ZIP → ZIPB без LLVM (zip-self/компилятор).

use std::path::{Path, PathBuf};
use std::process::Command;

fn корень() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("корень репозитория")
        .to_path_buf()
}

fn есть_clang() -> bool {
    Command::new("clang")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn exe(путь: &Path) -> PathBuf {
    if cfg!(windows) {
        путь.with_extension("exe")
    } else {
        путь.to_path_buf()
    }
}

#[test]
fn тест_файлы_компилятора_на_zip() {
    let к = корень();
    for rel in [
        "zip-self/компилятор.zip",
        "zip-self/байткод_эмиттер.zip",
        "zip-self/байткод_эмиттер_rt.c",
        "zip-self/компилятор_rt.c",
        "zip-self/compiler_rt.c",
        "zip-self/bytecode_emitter_rt.c",
        "docs/КОМПИЛЯТОР-НА-ZIP.md",
    ] {
        assert!(к.join(rel).is_file(), "нет файла {rel}");
    }
    let спека = std::fs::read_to_string(к.join("zip-self/компилятор.zip")).unwrap();
    assert!(спека.contains("#runtime"));
    assert!(спека.contains("компилятор_rt") || спека.contains("compiler_rt"));
}

#[test]
fn тест_компилятор_без_rust() {
    if !есть_clang() {
        eprintln!("clang не найден — пропуск сборки компилятора");
        return;
    }

    let к = корень();
    let out_dir = к.join("target/selfhost_test");
    let _ = std::fs::create_dir_all(&out_dir);

    let compiler_src = к.join("zip-self/compiler_rt.c");
    let compiler_bin = exe(&out_dir.join("компилятор_v1"));

    let статус = Command::new("clang")
        .args([
            compiler_src.to_str().unwrap(),
            "-O3",
            "-DZIP_COMPILER_MAIN",
            "-o",
        ])
        .arg(&compiler_bin)
        .status()
        .expect("clang");
    assert!(статус.success(), "не удалось собрать компилятор_v1");

    let hello = к.join("examples/привет_мир.zip");
    let zipb = out_dir.join("привет_self.zipb");
    let статус = Command::new(&compiler_bin)
        .args([hello.to_str().unwrap(), zipb.to_str().unwrap()])
        .status()
        .expect("запуск компилятора");
    assert!(статус.success(), "компилятор не собрал привет_мир.zip");

    let данные = std::fs::read(&zipb).expect("читать zipb");
    assert!(данные.len() >= 16);
    assert_eq!(&данные[0..4], b"ZIPB");
    assert_eq!(u16::from_le_bytes([данные[4], данные[5]]), 3);

    // Фиксированная точка: повторная компиляция даёт тот же ZIPB
    let zipb2 = out_dir.join("привет_self_2.zipb");
    let статус = Command::new(&compiler_bin)
        .args([hello.to_str().unwrap(), zipb2.to_str().unwrap()])
        .status()
        .expect("повторный запуск");
    assert!(статус.success());
    assert_eq!(
        std::fs::read(&zipb).unwrap(),
        std::fs::read(&zipb2).unwrap(),
        "ZIPB должен быть детерминированным"
    );

    let vm = exe(&к.join("zip-vm/target/release/zip-vm"));
    let vm_alt = exe(&к.join("zip-vm/target/debug/zip-vm"));
    let vm = if vm.is_file() {
        vm
    } else if vm_alt.is_file() {
        vm_alt
    } else {
        eprintln!("zip-vm не собран — пропуск запуска");
        return;
    };

    let выход = Command::new(&vm)
        .arg(&zipb)
        .output()
        .expect("zip-vm");
    assert!(выход.status.success(), "VM: {}", String::from_utf8_lossy(&выход.stderr));
    let stdout = String::from_utf8_lossy(&выход.stdout);
    assert!(
        stdout.contains("Привет"),
        "ожидался вывод Привет, получено: {stdout}"
    );
}
