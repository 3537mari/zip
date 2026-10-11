//! Инструмент верификации ZIP (`zip-verify`).

#![allow(confusable_idents)]

use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;
use zip_vm::байткод_v3_декодер::{разобрать_v3, это_zipb};
use zip_verify::{perfect, владение, типы};

#[derive(Parser)]
#[command(name = "zip-verify")]
#[command(about = "Формальная верификация байт-кода ZIP v3")]
struct Аргументы {
    /// Программа .zipb
    файл: PathBuf,

    /// Что проверять
    #[arg(long = "проверка", alias = "check", value_enum, default_value_t = Проверка::Все)]
    проверка: Проверка,

    /// Вывод отчёта в JSON
    #[arg(long)]
    json: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Проверка {
    #[value(name = "типы", alias = "types")]
    Типы,
    #[value(name = "владение", alias = "ownership")]
    Владение,
    #[value(name = "perfect", alias = "perfect-lock")]
    Perfect,
    #[default]
    #[value(name = "все", alias = "all")]
    Все,
}

#[derive(Serialize)]
struct ОтчётФункции {
    имя: String,
    типы_ok: Option<bool>,
    владение_ok: Option<bool>,
    perfect_ok: Option<bool>,
    ошибки: Vec<String>,
}

#[derive(Serialize)]
struct Отчёт {
    файл: String,
    функций: usize,
    ошибок: usize,
    функции: Vec<ОтчётФункции>,
}

fn имя_функции(строки: &[String], индекс: u32) -> String {
    строки
        .get(индекс as usize)
        .cloned()
        .unwrap_or_else(|| format!("#{индекс}"))
}

fn main() -> anyhow::Result<()> {
    let аргументы = Аргументы::parse();
    let данные = std::fs::read(&аргументы.файл)?;

    if !это_zipb(&данные) {
        anyhow::bail!("поддерживается только .zipb (магия ZIPB)");
    }

    let модуль = разобрать_v3(&данные).map_err(|е| anyhow::anyhow!("{е}"))?;

    println!("=== ВЕРИФИКАЦИЯ ZIP ===");
    println!("Файл: {}", аргументы.файл.display());
    println!("Функций: {}", модуль.функции.len());
    println!();

    let mut отчёт = Отчёт {
        файл: аргументы.файл.display().to_string(),
        функций: модуль.функции.len(),
        ошибок: 0,
        функции: Vec::new(),
    };

    let все = аргументы.проверка == Проверка::Все;

    for функция in &модуль.функции {
        let имя = имя_функции(&модуль.строки, функция.имя_индекс);
        println!("Проверка: {имя}");

        let mut ф = ОтчётФункции {
            имя: имя.clone(),
            типы_ok: None,
            владение_ok: None,
            perfect_ok: None,
            ошибки: Vec::new(),
        };

        if все || аргументы.проверка == Проверка::Типы {
            match типы::проверить_функцию(функция) {
                Ok(()) => {
                    println!("  ✓ Типы корректны");
                    ф.типы_ok = Some(true);
                }
                Err(е) => {
                    println!("  ✗ Ошибка типов: {е}");
                    ф.типы_ok = Some(false);
                    ф.ошибки.push(format!("типы: {е}"));
                    отчёт.ошибок += 1;
                }
            }
        }

        if все || аргументы.проверка == Проверка::Владение {
            match владение::проверить_владение(функция) {
                Ok(()) => {
                    println!("  ✓ Владение корректно");
                    ф.владение_ok = Some(true);
                }
                Err(е) => {
                    println!("  ✗ Ошибка владения: {е}");
                    ф.владение_ok = Some(false);
                    ф.ошибки.push(format!("владение: {е}"));
                    отчёт.ошибок += 1;
                }
            }
        }

        if все || аргументы.проверка == Проверка::Perfect {
            match perfect::проверить_perfect(функция) {
                Ok(()) => {
                    println!("  ✓ PERFECT-инвариант");
                    ф.perfect_ok = Some(true);
                }
                Err(е) => {
                    println!("  ✗ Ошибка PERFECT: {е}");
                    ф.perfect_ok = Some(false);
                    ф.ошибки.push(format!("perfect: {е}"));
                    отчёт.ошибок += 1;
                }
            }
        }

        println!();
        отчёт.функции.push(ф);
    }

    if let Some(путь) = &аргументы.json {
        let json = serde_json::to_string_pretty(&отчёт)?;
        std::fs::write(путь, json)?;
        println!("JSON: {}", путь.display());
    }

    if отчёт.ошибок == 0 {
        println!("✓ ВСЕ ПРОВЕРКИ ПРОЙДЕНЫ");
        Ok(())
    } else {
        println!("✗ Найдено ошибок: {}", отчёт.ошибок);
        std::process::exit(1);
    }
}
