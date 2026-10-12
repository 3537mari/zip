//! v6: объявление `структура` в C-bootstrap.

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
        return exe;
    }
    let unix = r.join("bootstrap/zipc");
    if !unix.exists() {
        let _ = Command::new("pwsh")
            .args(["-File", "bootstrap/build.ps1"])
            .current_dir(&r)
            .status();
    }
    if exe.exists() {
        exe
    } else {
        unix
    }
}

#[test]
fn тест_объявление_структуры() {
    let программа = r#"
структура Точка {
    х: Вещественное,
    у: Вещественное,
}

функция главная() {
    возврат(0)
}
"#;
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let файл = std::env::temp_dir().join(format!("тест_структура_{id}.zip"));
    let zipb = std::env::temp_dir().join(format!("тест_структура_{id}.zipb"));
    std::fs::write(&файл, программа).unwrap();
    let статус = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .output()
        .unwrap();
    assert!(
        статус.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&статус.stderr)
    );
    let bytes = std::fs::read(&zipb).unwrap();
    assert_eq!(&bytes[0..4], b"ZIPB");
}

#[test]
fn тест_несколько_полей() {
    let программа = r#"
структура Студент {
    имя: Строка,
    возраст: Целое,
    средний_балл: Вещественное,
}

функция главная() {
    возврат(42)
}
"#;
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let файл = std::env::temp_dir().join(format!("тест_студ_{id}.zip"));
    let zipb = std::env::temp_dir().join(format!("тест_студ_{id}.zipb"));
    std::fs::write(&файл, программа).unwrap();
    let статус = Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .status()
        .unwrap();
    assert!(статус.success());
}
