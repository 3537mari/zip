//! ЗИПМ — менеджер пакетов ZIP.

#[path = "компилятор.rs"]
mod компилятор;
#[path = "манифест.rs"]
mod манифест;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use компилятор::{путь_exe, скомпилировать};
use манифест::{записать, найти_корень_пакета, прочитать, Манифест, Сборка, Тесты};

#[derive(Parser)]
#[command(name = "зипм")]
#[command(version = "0.1.0")]
#[command(about = "Менеджер пакетов ZIP")]
struct Аргументы {
    #[command(subcommand)]
    команда: Команда,
}

#[derive(Subcommand)]
enum Команда {
    /// Создать новый пакет
    #[command(name = "создать", alias = "create")]
    Создать { имя: String },

    /// Собрать пакет
    #[command(name = "собрать", alias = "build")]
    Собрать {
        #[arg(long = "выпуск", alias = "release", default_value_t = false)]
        выпуск: bool,
    },

    /// Установить зависимости
    #[command(name = "установить", alias = "install")]
    Установить,

    /// Добавить зависимость
    #[command(name = "добавить", alias = "add")]
    Добавить {
        имя: String,
        #[arg(default_value = "latest")]
        версия: String,
    },

    /// Удалить зависимость
    #[command(name = "удалить", alias = "remove")]
    Удалить { имя: String },

    /// Опубликовать в реестр
    #[command(name = "опубликовать", alias = "publish")]
    Опубликовать,

    /// Обновить зависимости
    #[command(name = "обновить", alias = "update")]
    Обновить,

    /// Запустить программу
    #[command(name = "запустить", alias = "run")]
    Запустить { имя: Option<String> },

    /// Запустить тесты
    #[command(name = "тест", alias = "test")]
    Тест,

    /// Очистить кэш
    #[command(name = "очистить", alias = "clean")]
    Очистить,

    /// Показать информацию о пакете
    #[command(name = "инфо", alias = "info")]
    Инфо { имя: String },
}

fn main() -> Result<()> {
    let аргументы = Аргументы::parse();
    match аргументы.команда {
        Команда::Создать { имя } => создать_пакет(&имя),
        Команда::Собрать { выпуск } => собрать_пакет(выпуск),
        Команда::Установить => установить_зависимости(),
        Команда::Добавить { имя, версия } => добавить_зависимость(&имя, &версия),
        Команда::Удалить { имя } => удалить_зависимость(&имя),
        Команда::Опубликовать => опубликовать_пакет(),
        Команда::Обновить => обновить_зависимости(),
        Команда::Запустить { имя } => запустить_программу(имя),
        Команда::Тест => запустить_тесты(),
        Команда::Очистить => очистить_кэш(),
        Команда::Инфо { имя } => показать_инфо(&имя),
    }
}

fn создать_пакет(имя: &str) -> Result<()> {
    if имя.is_empty() || имя.contains(['/', '\\', ':']) {
        bail!("недопустимое имя пакета: {имя}");
    }
    println!("Создание пакета: {имя}");

    let корень = PathBuf::from(имя);
    if корень.exists() {
        bail!("каталог уже существует: {имя}");
    }

    fs::create_dir_all(корень.join("src"))?;
    fs::create_dir_all(корень.join("tests"))?;
    fs::create_dir_all(корень.join("examples"))?;

    let м = Манифест {
        имя: имя.to_string(),
        версия: "0.1.0".into(),
        автор: "Автор <email@example.com>".into(),
        лицензия: "MIT".into(),
        описание: format!("Пакет {имя}"),
        зависимости: Default::default(),
        зависимости_разработки: Default::default(),
        сборка: Сборка {
            тип: "программа".into(),
            точка_входа: "src/главная.zip".into(),
            выход: "цель/выпуск/программа".into(),
        },
        тесты: Тесты::default(),
    };
    записать(&корень, &м)?;

    let главная = "\
// Главная программа пакета\n\
печать_строки(\"Привет из ZIP-пакета!\")\n\
";
    fs::write(корень.join("src/главная.zip"), главная)?;
    fs::write(
        корень.join("tests/пример.zip"),
        "// тест-заглушка\nпечать_строки(\"тест ok\")\n",
    )?;
    fs::write(корень.join("README.md"), format!("# {имя}\n\nПакет ZIP.\n"))?;

    println!("✓ Пакет {имя} создан");
    println!("  Каталог: {имя}");
    println!("  Манифест: {имя}/зипм.манифест");
    println!("  Исходник: {имя}/src/главная.zip");
    Ok(())
}

fn собрать_пакет(выпуск: bool) -> Result<()> {
    let корень = найти_корень_пакета()?;
    let м = прочитать(&корень)?;
    println!(
        "Сборка пакета «{}» v{} (выпуск: {})",
        м.имя, м.версия, выпуск
    );

    let вход = корень.join(&м.сборка.точка_входа);
    if !вход.exists() {
        bail!("точка входа не найдена: {}", вход.display());
    }

    let выход = корень.join(&м.сборка.выход);
    скомпилировать(&вход, &выход, true)?;

    let exe = путь_exe(&выход);
    if exe.exists() {
        println!("✓ Собрано: {}", exe.display());
    } else {
        println!("✓ Сборка завершена: {}", выход.display());
    }
    Ok(())
}

fn путь_реестра() -> PathBuf {
    if let Ok(p) = std::env::var("ЗИПМ_РЕЕСТР").or_else(|_| std::env::var("ZIPM_REGISTRY")) {
        return PathBuf::from(p);
    }
    компилятор::корень_репозитория().join("реестр")
}

fn установить_зависимости() -> Result<()> {
    let корень = найти_корень_пакета()?;
    let м = прочитать(&корень)?;
    println!("Установка зависимостей...");

    let пакеты = корень.join("зипм_пакеты");
    fs::create_dir_all(&пакеты)?;

    let реестр = путь_реестра();
    if м.зависимости.is_empty() {
        println!("  (нет зависимостей в манифесте)");
    }
    for (имя, версия) in &м.зависимости {
        установить_одну(&реестр, &пакеты, имя, версия)?;
    }
    println!("✓ Зависимости установлены → {}", пакеты.display());
    Ok(())
}

fn установить_одну(реестр: &Path, пакеты: &Path, имя: &str, версия: &str) -> Result<()> {
    let вер = if версия == "latest" {
        найти_последнюю_версию(реестр, имя)?.unwrap_or_else(|| "0.1.0".into())
    } else {
        версия.to_string()
    };
    let источник = реестр.join("пакеты").join(имя).join(&вер);
    let назначение = пакеты.join(имя).join(&вер);
    if !источник.exists() {
        println!("  ! {имя}@{вер} — нет в локальном реестре ({})", источник.display());
        println!("    создаю заглушку-манифест");
        fs::create_dir_all(&назначение)?;
        let stub = Манифест {
            имя: имя.into(),
            версия: вер.clone(),
            ..Default::default()
        };
        записать(&назначение, &stub)?;
        return Ok(());
    }
    копировать_каталог(&источник, &назначение)?;
    println!("  + {имя}@{вер}");
    Ok(())
}

fn найти_последнюю_версию(реестр: &Path, имя: &str) -> Result<Option<String>> {
    let dir = реестр.join("пакеты").join(имя);
    if !dir.exists() {
        return Ok(None);
    }
    let mut vers: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    vers.sort();
    Ok(vers.pop())
}

fn копировать_каталог(из: &Path, в: &Path) -> Result<()> {
    fs::create_dir_all(в)?;
    for entry in fs::read_dir(из)? {
        let entry = entry?;
        let dest = в.join(entry.file_name());
        if entry.path().is_dir() {
            копировать_каталог(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

fn добавить_зависимость(имя: &str, версия: &str) -> Result<()> {
    let корень = найти_корень_пакета()?;
    let mut м = прочитать(&корень)?;
    println!("Добавление зависимости: {имя} = {версия}");
    м.зависимости.insert(имя.to_string(), версия.to_string());
    записать(&корень, &м)?;
    println!("✓ Зависимость добавлена в зипм.манифест");
    Ok(())
}

fn удалить_зависимость(имя: &str) -> Result<()> {
    let корень = найти_корень_пакета()?;
    let mut м = прочитать(&корень)?;
    println!("Удаление зависимости: {имя}");
    if м.зависимости.remove(имя).is_none() {
        bail!("зависимость «{имя}» не найдена");
    }
    записать(&корень, &м)?;
    println!("✓ Зависимость удалена");
    Ok(())
}

fn опубликовать_пакет() -> Result<()> {
    let корень = найти_корень_пакета()?;
    let м = прочитать(&корень)?;
    println!("Публикация пакета «{}» v{} в локальный реестр...", м.имя, м.версия);

    let цель = путь_реестра()
        .join("пакеты")
        .join(&м.имя)
        .join(&м.версия);
    fs::create_dir_all(&цель)?;

    // копируем исходники + манифест
    let src = корень.join("src");
    if src.exists() {
        копировать_каталог(&src, &цель.join("src"))?;
    }
    записать(&цель, &м)?;

    // checksum
    let манифест_путь = манифест::путь_манифеста(&цель);
    let bytes = fs::read(&манифест_путь)?;
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(&bytes);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    fs::write(цель.join("checksum.sha256"), format!("{hex}  зипм.манифест\n"))?;

    обновить_индекс_реестра(&м)?;
    println!("✓ Опубликовано: {}", цель.display());
    Ok(())
}

fn обновить_индекс_реестра(м: &Манифест) -> Result<()> {
    let индекс_путь = путь_реестра().join("index.toml");
    let mut текст = if индекс_путь.exists() {
        fs::read_to_string(&индекс_путь)?
    } else {
        String::from("# Индекс пакетов ZIP\n\n")
    };
    let блок = format!(
        "\n[пакеты.{}]\nверсии = [\"{}\"]\nописание = \"{}\"\n",
        м.имя, м.версия, м.описание
    );
    if !текст.contains(&format!("[пакеты.{}]", м.имя)) {
        текст.push_str(&блок);
        fs::create_dir_all(индекс_путь.parent().unwrap())?;
        fs::write(индекс_путь, текст)?;
    }
    Ok(())
}

fn обновить_зависимости() -> Result<()> {
    println!("Обновление зависимостей...");
    установить_зависимости()
}

fn запустить_программу(имя: Option<String>) -> Result<()> {
    let корень = найти_корень_пакета()?;
    let м = прочитать(&корень)?;
    let программа = if let Some(п) = имя {
        PathBuf::from(п)
    } else {
        путь_exe(&корень.join(&м.сборка.выход))
    };
    let программа = if программа.is_absolute() {
        программа
    } else {
        корень.join(&программа)
    };
    let программа = путь_exe(&программа);

    if !программа.exists() {
        println!("исполняемый файл не найден, собираю...");
        собрать_пакет(true)?;
    }
    let программа = путь_exe(&корень.join(&м.сборка.выход));
    println!("Запуск: {}", программа.display());
    let статус = Command::new(&программа)
        .current_dir(&корень)
        .status()
        .with_context(|| format!("запуск {}", программа.display()))?;
    if !статус.success() {
        bail!("программа завершилась с кодом {:?}", статус.code());
    }
    Ok(())
}

fn запустить_тесты() -> Result<()> {
    let корень = найти_корень_пакета()?;
    let м = прочитать(&корень)?;
    let каталог = корень.join(м.тесты.каталог.trim_end_matches('/'));
    println!("Запуск тестов из {}...", каталог.display());
    if !каталог.exists() {
        println!("  (каталог тестов отсутствует)");
        return Ok(());
    }
    let mut ok = 0usize;
    let mut fail = 0usize;
    for entry in fs::read_dir(&каталог)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("zip") {
            continue;
        }
        let out = корень.join("цель/тесты").join(path.file_stem().unwrap());
        print!("  {} ... ", path.file_name().unwrap().to_string_lossy());
        match скомпилировать(&path, &out, true) {
            Ok(()) => {
                let exe = путь_exe(&out);
                match Command::new(&exe).status() {
                    Ok(s) if s.success() => {
                        println!("ok");
                        ok += 1;
                    }
                    _ => {
                        println!("FAIL (run)");
                        fail += 1;
                    }
                }
            }
            Err(e) => {
                println!("FAIL ({e})");
                fail += 1;
            }
        }
    }
    println!("Итого: {ok} ok, {fail} fail");
    if fail > 0 {
        bail!("тесты не прошли");
    }
    Ok(())
}

fn очистить_кэш() -> Result<()> {
    let корень = найти_корень_пакета().unwrap_or_else(|_| std::env::current_dir().unwrap());
    println!("Очистка кэша в {}...", корень.display());
    let _ = fs::remove_dir_all(корень.join("цель"));
    let _ = fs::remove_dir_all(корень.join("зипм_пакеты"));
    println!("✓ Кэш очищен");
    Ok(())
}

fn показать_инфо(имя: &str) -> Result<()> {
    println!("Информация о пакете: {имя}");
    let реестр = путь_реестра().join("пакеты").join(имя);
    if !реестр.exists() {
        println!("  не найден в локальном реестре {}", реестр.display());
        return Ok(());
    }
    for entry in fs::read_dir(&реестр)? {
        let entry = entry?;
        if entry.path().is_dir() {
            let вер = entry.file_name().to_string_lossy().into_owned();
            let м_путь = манифест::путь_манифеста(&entry.path());
            if м_путь.exists() {
                let м = прочитать(&entry.path())?;
                println!("  {имя}@{вер} — {}", м.описание);
            } else {
                println!("  {имя}@{вер}");
            }
        }
    }
    Ok(())
}
