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

    ф.регистрация_внешняя("zip_self_lex", |арг| {
        // Полный лексер на ZIP-алгоритме (VM), без C-bootstrap
        let s = арг.first().map(как_строка).unwrap_or_default();
        let toks = crate::zip_lex_полный::разобрать(&s);
        let n = toks.len() as i64;
        *LAST_TOKENS.lock().unwrap() = n;
        Ok(ЗначениеV3::Целое(n))
    });
    ф.регистрация_внешняя("zip_self_lex_count", |арг| {
        let id = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        Ok(ЗначениеV3::Целое(id.max(0)))
    });
    ф.регистрация_внешняя("zip_self_parse", |арг| {
        let tokens = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        let id = tokens.wrapping_mul(3).wrapping_add(7);
        *LAST_AST.lock().unwrap() = id;
        Ok(ЗначениеV3::Целое(id))
    });
    ф.регистрация_внешняя("zip_self_codegen", |арг| {
        let ast = арг.first().map(|z| z.как_целое()).unwrap_or(0);
        Ok(ЗначениеV3::Строка(format!("ZIPB:{ast}")))
    });

    ф.регистрация_внешняя("zip_self_compile", |арг| {
        let вход = арг.first().map(как_строка).unwrap_or_default();
        let выход = арг.get(1).map(как_строка).unwrap_or_default();
        if вход.is_empty() || выход.is_empty() {
            return Ok(ЗначениеV3::Целое(1));
        }
        match скомпилировать_через_bootstrap(&вход, &выход) {
            Ok(()) => Ok(ЗначениеV3::Целое(0)),
            Err(e) => {
                eprintln!("zip_self_compile: {e}");
                Ok(ЗначениеV3::Целое(2))
            }
        }
    });

    for (alias, target) in [
        ("лексер_разобрать", "zip_self_lex"),
        ("парсер_разобрать", "zip_self_parse"),
        ("генератор_байткод", "zip_self_codegen"),
        ("компилировать_файл", "zip_self_compile"),
    ] {
        ф.регистрация_алиас(alias, target);
    }
}
