//! Эмиттер LLVM IR (текстовый .ll) + сборка через clang -O3.
//!
//! Применяет оптимизации LLVM (mem2reg, instcombine, simplifycfg,
//! loop-unroll, vectorize) через драйвер clang. Работает с winget LLVM
//! без полного SDK для llvm-sys/inkwell.

use crate::промежуток::{
    ИдентификаторФункции, Инструкция, РежимПечати, Регистр, ФункцияПП,
};
use std::collections::HashMap;
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

/// Сгенерировать текстовый LLVM IR (opaque pointers, LLVM 15+).
pub fn сгенерировать_llvm_ir(функции: &[ФункцияПП], имя_модуля: &str) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(&format!("; ModuleID = '{}'\n", ascii_name(имя_модуля)));
    out.push_str("source_filename = \"zip\"\n");
    out.push_str(
        "target datalayout = \"e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n",
    );
    out.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");

    out.push_str("declare i32 @puts(ptr)\n");
    out.push_str("declare i32 @printf(ptr, ...)\n\n");
    out.push_str("@.fmt_str = private unnamed_addr constant [3 x i8] c\"%s\\00\"\n");
    // "%lld\n\0" = 6 байт
    out.push_str("@.fmt_i64n = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n\n");

    let mut строки: Vec<String> = Vec::new();
    for ф in функции {
        for и in &ф.тело {
            if let Инструкция::КонстантаСтрока { значение, .. } = и {
                if !строки.iter().any(|s| s == значение) {
                    строки.push(значение.clone());
                }
            }
        }
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
    for ф in функции {
        имена.insert(ф.идентификатор, mangled(&ф.имя));
    }

    for ф in функции {
        out.push_str(&эмит_функцию(ф, &имена, &строки)?);
        out.push('\n');
    }

    Ok(out)
}

fn эмит_функцию(
    ф: &ФункцияПП,
    имена: &HashMap<ИдентификаторФункции, String>,
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

    let mut out = String::new();
    out.push_str(&format!("define i64 @{}() {{\n", fname));
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
            | Инструкция::Импульс {
                назначение,
                источник,
            } => {
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
                let op = match и {
                    Инструкция::Сложить { .. } => "add",
                    Инструкция::Вычесть { .. } => "sub",
                    Инструкция::Умножить { .. } => "mul",
                    Инструкция::Разделить { .. } => "sdiv",
                    _ => unreachable!(),
                };
                let t1 = next_tmp();
                let t2 = next_tmp();
                let t3 = next_tmp();
                out.push_str(&format!(
                    "  %t{} = load i64, ptr %r{}, align 8\n  %t{} = load i64, ptr %r{}, align 8\n  %t{} = {} i64 %t{}, %t{}\n  store i64 %t{}, ptr %r{}, align 8\n",
                    t1, а.0, t2, б.0, t3, op, t1, t2, t3, назначение.0
                ));
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
                        out.push_str(&format!(
                            "  call i32 (ptr, ...) @printf(ptr @.fmt_i64n, i64 %t{})\n",
                            t
                        ));
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
                результат,
                ..
            } => {
                let имя = имена
                    .get(функция)
                    .cloned()
                    .unwrap_or_else(|| format!("zip_fn_{}", функция.0));
                let t = next_tmp();
                out.push_str(&format!("  %t{} = call i64 @{}()\n", t, имя));
                if let Some(р) = результат {
                    out.push_str(&format!(
                        "  store i64 %t{}, ptr %r{}, align 8\n",
                        t, р.0
                    ));
                }
            }
            Инструкция::Выделить { назначение, .. } => {
                out.push_str(&format!(
                    "  store i64 0, ptr %r{}, align 8\n",
                    назначение.0
                ));
            }
            Инструкция::КонстантаВещественное { назначение, значение } => {
                out.push_str(&format!(
                    "  store i64 {}, ptr %r{}, align 8\n",
                    значение.to_bits() as i64,
                    назначение.0
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

/// Записать .ll и собрать через clang -O3.
pub fn скомпилировать_llvm(
    функции: &[ФункцияПП],
    имя_модуля: &str,
    выход: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let ir = сгенерировать_llvm_ir(функции, имя_модуля)?;
    let mut ll = выход.to_path_buf();
    ll.set_extension("ll");
    fs::write(&ll, &ir).map_err(|е| е.to_string())?;

    let mut exe = выход.to_path_buf();
    if cfg!(windows) && exe.extension().is_none() {
        exe.set_extension("exe");
    }

    let статус = Command::new("clang")
        .arg(&ll)
        .arg("-O3")
        .arg("-march=native")
        .arg("-o")
        .arg(&exe)
        .status()
        .map_err(|е| format!("clang: {}", е))?;

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
