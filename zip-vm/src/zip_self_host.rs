//! Самокомпиляция: маркеры совместимости + аргументы программы.
//! На native раньше вызывался C-bootstrap; в v6.1+ алгоритмы — в ZIP.

use crate::встроенные_v3::Встроенные;
use crate::исполнитель_v3::ЗначениеV3;
use std::sync::Mutex;

static PROG_ARGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
#[allow(dead_code)]
static LAST_TOKENS: Mutex<i64> = Mutex::new(0);
#[allow(dead_code)]
static LAST_AST: Mutex<i64> = Mutex::new(0);

/// Аргументы программы после пути к .zipb (для `получить_аргумент`).
pub fn установить_аргументы(args: Vec<String>) {
    *PROG_ARGS.lock().unwrap() = args;
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
}
