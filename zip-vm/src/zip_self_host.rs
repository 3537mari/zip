//! Самокомпиляция: ZIP-конвейер вызывает тот же C-bootstrap (fixed-point).

use crate::встроенные_v3::Встроенные;
use crate::исполнитель_v3::ЗначениеV3;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

static PROG_ARGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static LAST_TOKENS: Mutex<i64> = Mutex::new(0);
static LAST_AST: Mutex<i64> = Mutex::new(0);

/// Аргументы программы после пути к .zipb (для `получить_аргумент`).
pub fn установить_аргументы(args: Vec<String>) {
    *PROG_ARGS.lock().unwrap() = args;
}

fn как_строка(з: &ЗначениеV3) -> String {
    match з {
        ЗначениеV3::Строка(s) => s.clone(),
        другой => другой.to_string(),
    }
}

fn найти_корень() -> PathBuf {
    if let Ok(m) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(m).parent().unwrap_or(Path::new(".")).to_path_buf();
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn найти_zipc() -> Result<PathBuf, String> {
    let root = найти_корень();
    for p in [
        root.join("bootstrap/zipc.exe"),
        root.join("bootstrap/zipc"),
        PathBuf::from("bootstrap/zipc.exe"),
        PathBuf::from("bootstrap/zipc"),
    ] {
        if p.exists() {
            return Ok(p);
        }
    }
    Err("bootstrap/zipc не найден (собери: pwsh bootstrap/build.ps1)".into())
}

fn скомпилировать_через_bootstrap(вход: &str, выход: &str) -> Result<(), String> {
    let zipc = найти_zipc()?;
    let status = Command::new(&zipc)
        .args([вход, выход])
        .current_dir(найти_корень())
        .status()
        .map_err(|e| format!("запуск zipc: {e}"))?;
    if !status.success() {
        return Err(format!("zipc завершился с кодом {status}"));
    }
    Ok(())
}

pub fn зарегистрировать(ф: &mut Встроенные) {
    ф.регистрация_внешняя("количество_аргументов", |_арг| {
        let n = PROG_ARGS.lock().unwrap().len() as i64;
        Ok(ЗначениеV3::Целое(n))
    });
    ф.регистрация_внешняя("получить_аргумент", |арг| {
        let i = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        let args = PROG_ARGS.lock().unwrap();
        if i < 0 || i as usize >= args.len() {
            return Ok(ЗначениеV3::Строка(String::new()));
        }
        Ok(ЗначениеV3::Строка(args[i as usize].clone()))
    });
    ф.регистрация_внешняя("получить_аргументы", |_арг| {
        let n = PROG_ARGS.lock().unwrap().len() as i64;
        Ok(ЗначениеV3::Целое(n))
    });

    /* v6.1: алгоритмы компилятора — в ZIP (главное_чистое.zip).
     * Эти внешние имена — только маркеры совместимости, без работы. */
    ф.регистрация_внешняя("zip_self_lex", |_арг| Ok(ЗначениеV3::Целое(1)));
    ф.регистрация_внешняя("zip_self_lex_count", |_арг| Ok(ЗначениеV3::Целое(1)));
    ф.регистрация_внешняя("zip_self_parse", |_арг| Ok(ЗначениеV3::Целое(1)));
    ф.регистрация_внешняя("zip_self_codegen", |_арг| Ok(ЗначениеV3::Целое(1)));
    ф.регистрация_внешняя("zip_self_gen", |_арг| Ok(ЗначениеV3::Целое(1)));
    ф.регистрация_внешняя("zip_self_compile", |_арг| Ok(ЗначениеV3::Целое(1)));

    // v6.1: aliases removed — pure ZIP self-compiler must NOT call these.
    // лексер_разобрать, парсер_разобрать, генератор_байткод, компилировать_файл
    // are no longer registered; ZIP code uses its own lexer/parser/gen.
}
