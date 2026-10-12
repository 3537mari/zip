//! CLI виртуальной машины ZIP.

use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use zip_vm::байткод_v3_декодер::{разобрать_v3, это_zipb};
use zip_vm::исполнитель_v3::ИсполнительV3;
use zip_vm::{загрузить, Исполнитель};

#[derive(Parser, Debug)]
#[command(name = "zip-vm")]
#[command(version = "0.1.0")]
#[command(about = "Виртуальная машина байт-кода ZIP (.zipc / .zipb)")]
struct Аргументы {
    /// Режим генерации ZIPB из исходника `.zip` (VM-генератор, stdout)
    #[arg(long = "gen")]
    режим_генерации: bool,

    /// Файл байт-кода `.zipc`/`.zipb` или исходник `.zip` при `--gen`
    #[arg(value_name = "ФАЙЛ")]
    файл: Option<PathBuf>,

    /// Показать дамп модуля и выйти
    #[arg(long = "дамп")]
    дамп: bool,

    /// Принудительно использовать интерпретатор v3
    #[arg(long = "v3")]
    только_v3: bool,

    /// Аргументы программы (selfc: вход.zip выход.zipb)
    #[arg(value_name = "АРГ", trailing_var_arg = true, allow_hyphen_values = true)]
    программа: Vec<String>,
}

fn main() -> Result<()> {
    let аргументы = Аргументы::parse();

    if аргументы.режим_генерации {
        let входной = аргументы
            .файл
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--gen требует путь к .zip"))?;
        let исходник = std::fs::read_to_string(входной)
            .with_context(|| format!("чтение {}", входной.display()))?;
        match zip_vm::zip_gen_полный::сгенерировать_из_исходника(&исходник) {
            Ok(байткод) => {
                std::io::stdout().write_all(&байткод)?;
                Ok(())
            }
            Err(ошибка) => {
                eprintln!("Ошибка генерации: {ошибка}");
                std::process::exit(1);
            }
        }
    } else {
        let файл = аргументы
            .файл
            .ok_or_else(|| anyhow::anyhow!("укажите файл .zipb или --gen <файл.zip>"))?;

        let mut prog = vec![файл.display().to_string()];
        prog.extend(аргументы.программа.clone());
        zip_vm::zip_self_host::установить_аргументы(prog);

        let данные =
            std::fs::read(&файл).with_context(|| format!("чтение {}", файл.display()))?;

        if это_zipb(&данные) || аргументы.только_v3 {
            println!("=== ZIP VM v3 ===");
            println!("Файл: {}", файл.display());
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
            let модуль =
                загрузить(&файл).with_context(|| format!("загрузка {}", файл.display()))?;

            println!("=== ZIP-VM v2 ===");
            println!("Файл: {}", файл.display());
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
}
