//! CLI виртуальной машины ZIP.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use zip_vm::{загрузить, Исполнитель};

#[derive(Parser, Debug)]
#[command(name = "zip-vm")]
#[command(version = "0.1.0")]
#[command(about = "Виртуальная машина байт-кода ZIP (.zipc)")]
struct Аргументы {
    /// Файл байт-кода .zipc
    #[arg(value_name = "ФАЙЛ")]
    файл: PathBuf,

    /// Показать дамп модуля и выйти
    #[arg(long = "дамп")]
    дамп: bool,
}

fn main() -> Result<()> {
    let аргументы = Аргументы::parse();
    let модуль = загрузить(&аргументы.файл)
        .with_context(|| format!("загрузка {}", аргументы.файл.display()))?;

    println!("=== ZIP-VM v0.1.0 ===");
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
                println!("      {:04} {:?}", j, и);
            }
        }
        return Ok(());
    }

    let mut vm = Исполнитель::новый(модуль)?;
    match vm.выполнить() {
        Ok(значение) => {
            println!("✓ Результат: {}", значение);
            Ok(())
        }
        Err(е) => {
            eprintln!("✗ {}", е);
            std::process::exit(1);
        }
    }
}
