//! CLI виртуальной машины ZIP.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use zip_vm::байткод_v3_декодер::{разобрать_v3, это_zipb};
use zip_vm::исполнитель_v3::ИсполнительV3;
use zip_vm::{загрузить, Исполнитель};

#[derive(Parser, Debug)]
#[command(name = "zip-vm")]
#[command(version = "0.1.0")]
#[command(about = "Виртуальная машина байт-кода ZIP (.zipc / .zipb)")]
struct Аргументы {
    /// Файл байт-кода `.zipc` (v2) или `.zipb` (v3)
    #[arg(value_name = "ФАЙЛ")]
    файл: PathBuf,

    /// Показать дамп модуля и выйти
    #[arg(long = "дамп")]
    дамп: bool,

    /// Принудительно использовать интерпретатор v3
    #[arg(long = "v3")]
    только_v3: bool,
}

fn main() -> Result<()> {
    let аргументы = Аргументы::parse();
    let данные = std::fs::read(&аргументы.файл)
        .with_context(|| format!("чтение {}", аргументы.файл.display()))?;

    if это_zipb(&данные) || аргументы.только_v3 {
        println!("=== ZIP VM v3 ===");
        println!("Файл: {}", аргументы.файл.display());
        let модуль = разобрать_v3(&данные).map_err(|е| anyhow::anyhow!("{е}"))?;
        println!("Функций: {}", модуль.функции.len());
        println!("Строк: {}", модуль.строки.len());

        if аргументы.дамп {
            for (i, ф) in модуль.функции.iter().enumerate() {
                let имя = модуль
                    .строки
                    .get(ф.имя_индекс as usize)
                    .cloned()
                    .unwrap_or_else(|| format!("fn_{i}"));
                println!(
                    "  [{}] {} arity={} locals={} code_bytes={}",
                    i,
                    имя,
                    ф.арность,
                    ф.локалы,
                    ф.код.len()
                );
            }
            return Ok(());
        }

        let mut vm = ИсполнительV3::новый(модуль).map_err(|е| anyhow::anyhow!("{е}"))?;
        match vm.запустить() {
            Ok(результат) => {
                println!("✓ Результат: {результат}");
                Ok(())
            }
            Err(е) => {
                eprintln!("✗ Ошибка VM v3: {е}");
                std::process::exit(1);
            }
        }
    } else {
        let модуль = загрузить(&аргументы.файл)
            .with_context(|| format!("загрузка {}", аргументы.файл.display()))?;

        println!("=== ZIP-VM v2 ===");
        println!("Файл: {}", аргументы.файл.display());
        println!("Версия байт-кода: {}", модуль.версия);
        println!("Функций: {}", модуль.функции.len());
        println!("Строк: {}", модуль.строки.len());

        if аргументы.дамп {
            for (i, ф) in модуль.функции.iter().enumerate() {
                println!(
                    "  [{}] {} params={} locals={} code={}",
                    i,
                    ф.имя,
                    ф.параметры,
                    ф.локальные,
                    ф.код.len()
                );
                for (j, и) in ф.код.iter().enumerate() {
                    println!("      {j:04} {и:?}");
                }
            }
            return Ok(());
        }

        let mut vm = Исполнитель::новый(модуль)?;
        match vm.выполнить() {
            Ok(значение) => {
                println!("✓ Результат: {значение}");
                Ok(())
            }
            Err(е) => {
                eprintln!("✗ {е}");
                std::process::exit(1);
            }
        }
    }
}
