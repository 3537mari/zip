//! v6: массивы в C-bootstrap (литералы, индекс, многомерные).

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

fn скомпилировать(программа: &str) -> bool {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let файл = std::env::temp_dir().join(format!("тест_массив_{}.zip", id));
    let zipb = std::env::temp_dir().join(format!("тест_массив_{}.zipb", id));

    std::fs::write(&файл, программа).unwrap();

    Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .status()
        .map(|с| с.success())
        .unwrap_or(false)
}

#[test]
fn тест_литерал_массива() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            числа = [1, 2, 3, 4, 5]
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_повтор_массива() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            нули = [0; 10]
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_доступ_по_индексу() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            числа = [1, 2, 3, 4, 5]
            значение = числа[0]
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_запись_по_индексу() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            числа = [1, 2, 3]
            числа[0] = 10
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_многомерный_массив() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            матрица = [[1, 2], [3, 4]]
            значение = матрица[0][1]
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_массив_структур() {
    assert!(скомпилировать(
        r#"
        структура Точка {
            х: Вещественное,
            у: Вещественное,
        }

        функция главная() {
            точки = [
                Точка { х: 0.0, у: 0.0 },
                Точка { х: 1.0, у: 1.0 },
            ]
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_длина_массива() {
    assert!(скомпилировать(
        r#"
        функция главная() {
            числа = [1, 2, 3, 4, 5]
            длина = длина_массива(числа)
            возврат(0)
        }
    "#
    ));
}
