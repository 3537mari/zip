//! Эмиттер LLVM IR (текстовый .ll) + сборка через clang -O3.
//!
//! Применяет оптимизации LLVM (mem2reg, instcombine, simplifycfg,
//! loop-unroll, vectorize) через драйвер clang. Работает с winget LLVM
//! без полного SDK для llvm-sys/inkwell.

use crate::промежуток::{
    ИдентификаторФункции, Инструкция, РежимПечати, Регистр, ФункцияПП, Тип,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Оптимизационные проходы clang -O3 / LLVM.
pub const LLVM_PASSES: &[&str] = &[
    "mem2reg",
    "instcombine",
    "simplifycfg",
    "loop-unroll",
    "loop-vectorize",
    "slp-vectorizer",
];

/// Явные SIMD-проходы (pipeline string для документации / opt).
pub const LLVM_PASSES_SIMD: &str = "vectorize[vectorize-loops,vectorize-slp],unroll-loops";

/// Сгенерировать текстовый LLVM IR (opaque pointers, LLVM 15+).
pub fn сгенерировать_llvm_ir(функции: &[ФункцияПП], имя_модуля: &str) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(&format!("; ModuleID = '{}'\n", ascii_name(имя_модуля)));
    out.push_str("source_filename = \"zip\"\n");
    out.push_str(
        "target datalayout = \"e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n",
    );
    out.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");
    out.push_str("; Проходы: mem2reg, instcombine, simplifycfg, ");
    out.push_str("loop-unroll, loop-vectorize, slp-vectorizer\n");
    out.push_str(&format!("; SIMD-проходы: {}\n", LLVM_PASSES_SIMD));
    out.push_str("; Для горячих функций: aggressive-inline, ");
    out.push_str("loop-unroll-full, vectorize-width=512\n\n");

    out.push_str("declare i32 @puts(ptr)\n");
    out.push_str("declare i32 @printf(ptr, ...)\n");
    out.push_str("declare ptr @malloc(i64)\n");
    out.push_str("declare i64 @strlen(ptr)\n");
    out.push_str("declare ptr @memcpy(ptr, ptr, i64)\n");
    out.push_str("declare void @exit(i32)\n\n");
    out.push_str("@.fmt_str = private unnamed_addr constant [3 x i8] c\"%s\\00\"\n");
    // "%lld\n\0" = 6 байт
    out.push_str("@.fmt_i64n = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n");
    // "%.1f\n\0" — вещественные с одним знаком после запятой (демо: 10.0)
    out.push_str("@.fmt_f64n = private unnamed_addr constant [6 x i8] c\"%.1f\\0A\\00\"\n");
    // "type error: no matching specialization\n\0" = 40 байт
    out.push_str("@.fmt_type_err = private unnamed_addr constant [40 x i8] c\"type error: no matching specialization\\0A\\00\"\n\n");

    let mut строки: Vec<String> = Vec::new();
    let mut глобалы: Vec<String> = Vec::new();
    for ф in функции {
        for и in &ф.тело {
            match и {
                Инструкция::КонстантаСтрока { значение, .. } => {
                    if !строки.iter().any(|s| s == значение) {
                        строки.push(значение.clone());
                    }
                }
                Инструкция::ЗагрузитьГлобал { имя, .. }
                | Инструкция::СохранитьГлобал { имя, .. } => {
                    if !глобалы.iter().any(|g| g == имя) {
                        глобалы.push(имя.clone());
                    }
                }
                _ => {}
            }
        }
    }
    for имя in &глобалы {
        out.push_str(&format!(
            "@{} = global i64 0, align 8\n",
            mangled(имя)
        ));
    }
    if !глобалы.is_empty() {
        out.push('\n');
    }
    for (i, s) in строки.iter().enumerate() {
        let bytes = escape_llvm_string(s);
        let len = s.len() + 1;
        out.push_str(&format!(
            "@.str{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
            i, len, bytes
        ));
    }
    if !строки.is_empty() {
        out.push('\n');
    }

    let mut имена: HashMap<ИдентификаторФункции, String> = HashMap::new();
    let mut типы_возврата: HashMap<ИдентификаторФункции, Тип> = HashMap::new();
    for ф in функции {
        имена.insert(ф.идентификатор, mangled(&ф.имя));
        типы_возврата.insert(ф.идентификатор, ф.тип_возврата.clone());
    }

    let есть_горячие = функции
        .iter()
        .any(|ф| ф.агрессивная_оптимизация || ф.уже_горячая);
    let есть_шифр = функции.iter().any(|ф| ф.шифровать);

    for ф in функции {
        out.push_str(&эмит_функцию(ф, &имена, &типы_возврата, &строки)?);
        out.push('\n');
    }

    if есть_горячие {
        out.push_str(
            "; Пластичность: hot + noinline (агрессивные проходы: loop-unroll-full, vectorize)\n",
        );
        out.push_str("attributes #0 = { hot noinline }\n");
    }
    if есть_шифр {
        out.push_str("; Эпигенетика @шифровать: optnone (не ломать крипто)\n");
        out.push_str("attributes #1 = { optnone noinline }\n");
    }

    Ok(out)
}

fn эмит_функцию(
    ф: &ФункцияПП,
    имена: &HashMap<ИдентификаторФункции, String>,
    типы_возврата: &HashMap<ИдентификаторФункции, Тип>,
    строки: &[String],
) -> Result<String, String> {
    let имя = имена
        .get(&ф.идентификатор)
        .cloned()
        .unwrap_or_else(|| mangled(&ф.имя));
    let fname = if ф.имя == "главная" {
        "main".to_string()
    } else {
        имя
    };

    // R3: runtime-хелперы со специальной реализацией
    if ф.имя == "соединить_строки" || ф.имя == "zip_соединить_строки" {
        return Ok(эмит_соединить_строки(&fname));
    }
    if ф.имя == "zip_ошибка_типа" {
        return Ok(эмит_ошибка_типа(&fname));
    }
    if ф.имя == "zip_агент_отправить" {
        return Ok(эмит_агент_отправить(&fname));
    }

    let mut out = String::new();
    // Сигнатура с параметрами (i64…)
    let params_sig: Vec<String> = ф
        .параметры
        .iter()
        .enumerate()
        .map(|(i, _)| format!("i64 %arg{i}"))
        .collect();
    // Идея 4 / 2: атрибуты hot / optnone (эпигенетика)
    let attrs = if ф.шифровать {
        " #1"
    } else if ф.агрессивная_оптимизация || ф.уже_горячая {
        " #0"
    } else {
        ""
    };
    if ф.агрессивная_оптимизация || ф.уже_горячая {
        out.push_str(&format!(
            "; hot function «{}» (порог={}, агрессивная оптимизация)\n",
            ф.имя, ф.порог_горячности
        ));
    }
    // Идея 2 — метаданные эпигенетики
    out.push_str(&format!(
        "; оптимизация: уровень {}\n",
        ф.уровень_оптимизации
    ));
    if ф.кэшировать {
        out.push_str(&format!("; кэш: {} элементов\n", ф.размер_кэша));
    }
    if ф.шифровать {
        out.push_str(&format!("; шифрование: {}\n", ф.алгоритм_шифрования));
    }
    if ф.параллелить {
        out.push_str(&format!(
            "; параллелизм: {} потоков\n",
            ф.количество_потоков
        ));
    }
    if ф.сон {
        out.push_str(&format!("; сон: {}\n", ф.интервал_сна));
    }
    out.push_str(&format!(
        "define i64 @{}({}){} {{\n",
        fname,
        params_sig.join(", "),
        attrs
    ));
    out.push_str("entry:\n");

    let max_r = ф
        .тело
        .iter()
        .flat_map(regs_of)
        .chain(ф.параметры.iter().copied())
        .map(|r| r.0)
        .max()
        .unwrap_or(0);
    for i in 0..=max_r {
        out.push_str(&format!("  %r{} = alloca i64, align 8\n", i));
    }
    // Скопировать аргументы в регистры-параметры
    for (i, р) in ф.параметры.iter().enumerate() {
        out.push_str(&format!(
            "  store i64 %arg{i}, ptr %r{}, align 8\n",
            р.0
        ));
    }

    let mut float_regs: HashSet<u32> = HashSet::new();
    if matches!(ф.тип_возврата, Тип::Вещественное64) {
        // параметры вещественного варианта специализации
        for р in &ф.параметры {
            float_regs.insert(р.0);
        }
    }

    let mut str_map: HashMap<String, usize> = HashMap::new();
    for (i, s) in строки.iter().enumerate() {
        str_map.insert(s.clone(), i);
    }

    let mut tmp = 0u32;
    let mut next_tmp = || {
        let t = tmp;
        tmp += 1;
        t
    };

    for и in &ф.тело {
        match и {
            Инструкция::КонстантаЦелое { назначение, значение } => {
                out.push_str(&format!(
                    "  store i64 {}, ptr %r{}, align 8\n",
                    значение, назначение.0
                ));
            }
            Инструкция::КонстантаСтрока {
                назначение,
                значение,
            } => {
                let idx = str_map.get(значение).copied().unwrap_or(0);
                let len = значение.len() + 1;
                let t = next_tmp();
                out.push_str(&format!(
                    "  %t{} = getelementptr inbounds [{} x i8], ptr @.str{}, i64 0, i64 0\n",
                    t, len, idx
                ));
                out.push_str(&format!(
                    "  %t{0}_p = ptrtoint ptr %t{0} to i64\n  store i64 %t{0}_p, ptr %r{1}, align 8\n",
                    t, назначение.0
                ));
            }
            Инструкция::Переместить {
                назначение,
                источник,
            }
            | Инструкция::Копировать {
                назначение,
                источник,
            }
            | Инструкция::Импульс {
                назначение,
                источник,
            } => {
                if float_regs.contains(&источник.0) {
                    float_regs.insert(назначение.0);
                }
                let t = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr %r{}, align 8\n  store i64 %t{}, ptr %r{}, align 8\n",
                    t, источник.0, t, назначение.0
                ));
            }
            Инструкция::Сложить { назначение, а, б }
            | Инструкция::Вычесть { назначение, а, б }
            | Инструкция::Умножить { назначение, а, б }
            | Инструкция::Разделить { назначение, а, б } => {
                let as_float =
                    float_regs.contains(&а.0) || float_regs.contains(&б.0);
                let t1 = next_tmp();
                let t2 = next_tmp();
                let t3 = next_tmp();
                if as_float {
                    float_regs.insert(назначение.0);
                    let fop = match и {
                        Инструкция::Сложить { .. } => "fadd",
                        Инструкция::Вычесть { .. } => "fsub",
                        Инструкция::Умножить { .. } => "fmul",
                        Инструкция::Разделить { .. } => "fdiv",
                        _ => unreachable!(),
                    };
                    let fa = next_tmp();
                    let fb = next_tmp();
                    out.push_str(&format!(
                        "  %t{} = load i64, ptr %r{}, align 8\n  %t{} = load i64, ptr %r{}, align 8\n  %t{} = bitcast i64 %t{} to double\n  %t{} = bitcast i64 %t{} to double\n  %t{} = {} double %t{}, %t{}\n  %t{}_b = bitcast double %t{} to i64\n  store i64 %t{}_b, ptr %r{}, align 8\n",
                        t1, а.0, t2, б.0, fa, t1, fb, t2, t3, fop, fa, fb, t3, t3, t3, назначение.0
                    ));
                } else {
                    let op = match и {
                        Инструкция::Сложить { .. } => "add",
                        Инструкция::Вычесть { .. } => "sub",
                        Инструкция::Умножить { .. } => "mul",
                        Инструкция::Разделить { .. } => "sdiv",
                        _ => unreachable!(),
                    };
                    out.push_str(&format!(
                        "  %t{} = load i64, ptr %r{}, align 8\n  %t{} = load i64, ptr %r{}, align 8\n  %t{} = {} i64 %t{}, %t{}\n  store i64 %t{}, ptr %r{}, align 8\n",
                        t1, а.0, t2, б.0, t3, op, t1, t2, t3, назначение.0
                    ));
                }
            }
            Инструкция::Сравнить {
                оператор,
                назначение,
                а,
                б,
            } => {
                let pred = match оператор.as_str() {
                    "==" => "eq",
                    "!=" => "ne",
                    "<" => "slt",
                    ">" => "sgt",
                    "<=" => "sle",
                    ">=" => "sge",
                    _ => "ne",
                };
                let t1 = next_tmp();
                let t2 = next_tmp();
                let t3 = next_tmp();
                let t4 = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr %r{}, align 8\n  %t{} = load i64, ptr %r{}, align 8\n  %t{} = icmp {} i64 %t{}, %t{}\n  %t{} = zext i1 %t{} to i64\n  store i64 %t{}, ptr %r{}, align 8\n",
                    t1, а.0, t2, б.0, t3, pred, t1, t2, t4, t3, t4, назначение.0
                ));
            }
            Инструкция::Печать { режим, значение } => {
                let t = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr %r{}, align 8\n",
                    t, значение.0
                ));
                match режим {
                    РежимПечати::СоСтрокой => {
                        let tp = next_tmp();
                        out.push_str(&format!(
                            "  %t{} = inttoptr i64 %t{} to ptr\n  call i32 @puts(ptr %t{})\n",
                            tp, t, tp
                        ));
                    }
                    РежимПечати::БезНовойСтроки => {
                        let tp = next_tmp();
                        out.push_str(&format!(
                            "  %t{} = inttoptr i64 %t{} to ptr\n  call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr %t{})\n",
                            tp, t, tp
                        ));
                    }
                    РежимПечати::Число => {
                        if float_regs.contains(&значение.0) {
                            let tf = next_tmp();
                            out.push_str(&format!(
                                "  %t{} = bitcast i64 %t{} to double\n  call i32 (ptr, ...) @printf(ptr @.fmt_f64n, double %t{})\n",
                                tf, t, tf
                            ));
                        } else {
                            out.push_str(&format!(
                                "  call i32 (ptr, ...) @printf(ptr @.fmt_i64n, i64 %t{})\n",
                                t
                            ));
                        }
                    }
                }
            }
            Инструкция::Метка { идентификатор } => {
                out.push_str(&format!("L{}:\n", идентификатор.0));
            }
            Инструкция::Перейти { метка } => {
                out.push_str(&format!("  br label %L{}\n", метка.0));
            }
            Инструкция::ВетвлениеИмпульса {
                условие,
                истина,
                ложь,
            } => {
                let t = next_tmp();
                let tc = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr %r{}, align 8\n  %t{} = icmp ne i64 %t{}, 0\n  br i1 %t{}, label %L{}, label %L{}\n",
                    t, условие.0, tc, t, tc, истина.0, ложь.0
                ));
            }
            Инструкция::Вернуть { значение } => {
                if let Some(р) = значение {
                    let t = next_tmp();
                    out.push_str(&format!(
                        "  %t{} = load i64, ptr %r{}, align 8\n  ret i64 %t{}\n",
                        t, р.0, t
                    ));
                } else {
                    out.push_str("  ret i64 0\n");
                }
            }
            Инструкция::Вызвать {
                функция,
                аргументы,
                результат,
                ..
            } => {
                let имя = имена
                    .get(функция)
                    .cloned()
                    .unwrap_or_else(|| format!("zip_fn_{}", функция.0));
                let mut loaded = Vec::new();
                for арг in аргументы {
                    let t = next_tmp();
                    out.push_str(&format!(
                        "  %t{} = load i64, ptr %r{}, align 8\n",
                        t, арг.0
                    ));
                    loaded.push(format!("i64 %t{t}"));
                }
                let t = next_tmp();
                out.push_str(&format!(
                    "  %t{} = call i64 @{}({})\n",
                    t,
                    имя,
                    loaded.join(", ")
                ));
                if let Some(р) = результат {
                    out.push_str(&format!(
                        "  store i64 %t{}, ptr %r{}, align 8\n",
                        t, р.0
                    ));
                    if matches!(
                        типы_возврата.get(функция),
                        Some(Тип::Вещественное64)
                    ) {
                        float_regs.insert(р.0);
                    }
                }
            }
            Инструкция::Выделить { назначение, .. } => {
                out.push_str(&format!(
                    "  store i64 0, ptr %r{}, align 8\n",
                    назначение.0
                ));
            }
            Инструкция::КонстантаВещественное { назначение, значение } => {
                float_regs.insert(назначение.0);
                out.push_str(&format!(
                    "  store i64 {}, ptr %r{}, align 8\n",
                    значение.to_bits() as i64,
                    назначение.0
                ));
            }
            Инструкция::ЗагрузитьГлобал { назначение, имя } => {
                let t = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr @{}, align 8\n  store i64 %t{}, ptr %r{}, align 8\n",
                    t,
                    mangled(имя),
                    t,
                    назначение.0
                ));
            }
            Инструкция::СохранитьГлобал { имя, значение } => {
                let t = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr %r{}, align 8\n  store i64 %t{}, ptr @{}, align 8\n",
                    t,
                    значение.0,
                    t,
                    mangled(имя)
                ));
            }
            _ => out.push_str("  ; stub\n"),
        }
    }

    if !ф.тело.iter().any(|и| matches!(и, Инструкция::Вернуть { .. })) {
        out.push_str("  ret i64 0\n");
    }
    out.push_str("}\n");
    Ok(out)
}

fn regs_of(и: &Инструкция) -> Vec<Регистр> {
    match и {
        Инструкция::КонстантаЦелое { назначение, .. }
        | Инструкция::КонстантаВещественное { назначение, .. }
        | Инструкция::КонстантаСтрока { назначение, .. }
        | Инструкция::Выделить { назначение, .. } => vec![*назначение],
        Инструкция::Печать { значение, .. } => vec![*значение],
        Инструкция::Переместить {
            назначение,
            источник,
        }
        | Инструкция::Копировать {
            назначение,
            источник,
        }
        | Инструкция::Импульс {
            назначение,
            источник,
        } => vec![*назначение, *источник],
        Инструкция::Сложить { назначение, а, б }
        | Инструкция::Вычесть { назначение, а, б }
        | Инструкция::Умножить { назначение, а, б }
        | Инструкция::Разделить { назначение, а, б }
        | Инструкция::Сравнить {
            назначение,
            а,
            б,
            ..
        } => vec![*назначение, *а, *б],
        Инструкция::ВетвлениеИмпульса { условие, .. } => vec![*условие],
        Инструкция::Вернуть { значение } => значение.iter().copied().collect(),
        Инструкция::Вызвать { результат, .. } => результат.iter().copied().collect(),
        Инструкция::ЗагрузитьГлобал { назначение, .. } => vec![*назначение],
        Инструкция::СохранитьГлобал { значение, .. } => vec![*значение],
        _ => vec![],
    }
}

fn mangled(имя: &str) -> String {
    if имя == "главная" {
        "zip_glavnaya".into()
    } else {
        format!(
            "zip_{}",
            имя.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_string()
                    } else {
                        format!("u{:04x}", c as u32)
                    }
                })
                .collect::<String>()
        )
    }
}

/// R3: конкатенация двух C-строк (указатели как i64).
fn эмит_соединить_строки(fname: &str) -> String {
    format!(
        r#"define i64 @{fname}(i64 %arg0, i64 %arg1) {{
entry:
  %pa = inttoptr i64 %arg0 to ptr
  %pb = inttoptr i64 %arg1 to ptr
  %la = call i64 @strlen(ptr %pa)
  %lb = call i64 @strlen(ptr %pb)
  %sum = add i64 %la, %lb
  %n = add i64 %sum, 1
  %buf = call ptr @malloc(i64 %n)
  %_ = call ptr @memcpy(ptr %buf, ptr %pa, i64 %la)
  %dst = getelementptr i8, ptr %buf, i64 %la
  %__ = call ptr @memcpy(ptr %dst, ptr %pb, i64 %lb)
  %end = getelementptr i8, ptr %buf, i64 %sum
  store i8 0, ptr %end, align 1
  %ret = ptrtoint ptr %buf to i64
  ret i64 %ret
}}
"#
    )
}

/// R3: runtime-ошибка специализации.
fn эмит_ошибка_типа(fname: &str) -> String {
    format!(
        r#"define i64 @{fname}(i64 %arg0) {{
entry:
  call i32 (ptr, ...) @printf(ptr @.fmt_type_err)
  call void @exit(i32 1)
  unreachable
}}
"#
    )
}

/// Идея 17: печать сообщения агента (указатель строки как i64).
fn эмит_агент_отправить(fname: &str) -> String {
    format!(
        r#"define i64 @{fname}(i64 %arg0) {{
entry:
  %p = inttoptr i64 %arg0 to ptr
  call i32 @puts(ptr %p)
  ret i64 0
}}
"#
    )
}

fn ascii_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_llvm_string(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match *b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\22"),
            b'\n' => out.push_str("\\0A"),
            0x20..=0x7e => out.push(*b as char),
            _ => out.push_str(&format!("\\{:02X}", b)),
        }
    }
    out
}

/// Записать .ll и собрать через clang -O3 (-march=native).
pub fn скомпилировать_llvm(
    функции: &[ФункцияПП],
    имя_модуля: &str,
    выход: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    скомпилировать_llvm_с_опциями(функции, имя_модуля, выход, false)
}

/// Сборка LLVM IR с опциональными явными SIMD-флагами (`--simd`).
pub fn скомпилировать_llvm_с_опциями(
    функции: &[ФункцияПП],
    имя_модуля: &str,
    выход: &Path,
    simd: bool,
) -> Result<(PathBuf, PathBuf), String> {
    let ir = сгенерировать_llvm_ir(функции, имя_модуля)?;
    let mut ll = выход.to_path_buf();
    ll.set_extension("ll");
    fs::write(&ll, &ir).map_err(|е| е.to_string())?;

    let mut exe = выход.to_path_buf();
    if cfg!(windows) && exe.extension().is_none() {
        exe.set_extension("exe");
    }

    let mut cmd = Command::new("clang");
    cmd.arg(&ll).arg("-O3").arg("-march=native");
    if simd {
        for флаг in crate::simd::флаги_clang_simd() {
            if флаг != "-march=native" {
                cmd.arg(флаг);
            }
        }
    }
    cmd.arg("-o").arg(&exe);

    let статус = cmd.status().map_err(|е| format!("clang: {}", е))?;

    if статус.success() {
        Ok((exe, ll))
    } else {
        Err(format!(
            "clang -O3 код {:?} (IR: {})",
            статус.code(),
            ll.display()
        ))
    }
}
