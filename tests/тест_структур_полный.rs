//! v6: полная поддержка structs в C-bootstrap (литералы, поля, вложенность).

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

    let файл = std::env::temp_dir().join(format!("тест_стр_{}.zip", id));
    let zipb = std::env::temp_dir().join(format!("тест_стр_{}.zipb", id));

    std::fs::write(&файл, программа).unwrap();

    Command::new(bootstrap())
        .args([файл.to_str().unwrap(), zipb.to_str().unwrap()])
        .current_dir(корень())
        .status()
        .map(|с| с.success())
        .unwrap_or(false)
}

#[test]
fn тест_объявление_структуры() {
    assert!(скомпилировать(
        r#"
        структура Точка {
            х: Вещественное,
            у: Вещественное,
        }
    "#
    ));
}

#[test]
fn тест_литерал_структуры() {
    assert!(скомпилировать(
        r#"
        структура Точка {
            х: Вещественное,
            у: Вещественное,
        }

        функция главная() {
            точка = Точка { х: 1.0, у: 2.0 }
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_доступ_к_полю() {
    assert!(скомпилировать(
        r#"
        структура Точка {
            х: Вещественное,
            у: Вещественное,
        }

        функция главная() {
            точка = Точка { х: 1.0, у: 2.0 }
            значение = точка.х
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_вложенная_структура() {
    assert!(скомпилировать(
        r#"
        структура Точка {
            х: Вещественное,
            у: Вещественное,
        }

        структура Линия {
            начало: Точка,
            конец: Точка,
        }

        функция главная() {
            линия = Линия {
                начало: Точка { х: 0.0, у: 0.0 },
                конец: Точка { х: 1.0, у: 1.0 },
            }
            возврат(0)
        }
    "#
    ));
}

#[test]
fn тест_много_полей() {
    assert!(скомпилировать(
        r#"
        структура Студент {
            имя: Строка,
            возраст: Целое,
            средний_балл: Вещественное,
            курс: Целое,
            группа: Строка,
        }

        функция главная() {
            с = Студент {
                имя: "Иван",
                возраст: 20,
                средний_балл: 4.5,
                курс: 3,
                группа: "ИУ7-31Б",
            }
            возврат(0)
        }
    "#
    ));
}
