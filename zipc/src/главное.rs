//! Точка входа компилятора ZIP (`zipc`).

// Bin дублирует модули lib через `#[path]` — не все символы используются в CLI.
#![allow(dead_code)]
#![allow(confusable_idents)]

#[path = "лексика.rs"]
mod лексика;
#[path = "дерево.rs"]
mod дерево;
#[path = "парсер.rs"]
mod парсер;
#[path = "промежуток.rs"]
mod промежуток;
#[path = "нативный.rs"]
mod нативный;
#[path = "llvm_ir.rs"]
mod llvm_ir;
#[path = "байткод_эмиттер.rs"]
mod байткод_эмиттер;
#[path = "владение.rs"]
mod владение;
#[path = "компилятор_времени.rs"]
mod компилятор_времени;
#[path = "комплементарность.rs"]
mod комплементарность;
#[path = "аннотации.rs"]
mod аннотации;
#[path = "ассоциации.rs"]
mod ассоциации;
#[path = "корни.rs"]
mod корни;
#[path = "генератор_тестов.rs"]
mod генератор_тестов;
#[path = "нейрогенез.rs"]
mod нейрогенез;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use дерево::Узел;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "zipc")]
#[command(version = "0.1.0")]
#[command(about = "Компилятор языка ZIP (Zero-Immunity-Pulse)")]
#[command(subcommand_negates_reqs = true)]
struct Аргументы {
    #[command(subcommand)]
    команда: Option<Команда>,

    /// Входной файл .zip (старый режим без подкоманды)
    #[arg(value_name = "ФАЙЛ")]
    входной_файл: Option<PathBuf>,

    /// Выходной файл (без расширения)
    #[arg(short = 'o', long = "выход", default_value = "выход")]
    выходной_файл: PathBuf,

    /// Только лексический анализ
    #[arg(long = "только-лексика", alias = "lex-only")]
    только_лексика: bool,

    /// Только парсинг (вывод AST)
    #[arg(long = "только-парсинг", alias = "parse-only")]
    только_парсинг: bool,

    /// Вывод промежуточного представления
    #[arg(long = "вывод-пп", alias = "dump-ir")]
    вывод_пп: bool,

    /// Вывод ассемблера (.s) и останов
    #[arg(long = "вывод-ассемблера")]
    вывод_ассемблера: bool,

    /// LLVM-бэкенд: эмит .ll + clang -O3 (mem2reg/instcombine/vectorize)
    #[arg(long = "llvm")]
    llvm: bool,

    /// Только вывести LLVM IR (.ll) и останов
    #[arg(long = "вывод-llvm")]
    вывод_llvm: bool,

    /// Сгенерировать байт-код .zipc для zip-vm
    #[arg(long = "байткод", alias = "bytecode")]
    байткод: bool,

    /// Отключить анализ владения импульсов (R1 / Polonius)
    #[arg(long = "без-владения", alias = "no-ownership")]
    без_владения: bool,

    /// Идея 20: сгенерировать и запустить автотесты `@тест`
    #[arg(long = "запустить-тесты", alias = "run-tests")]
    запустить_тесты: bool,

    /// Уровень оптимизации 0–3
    #[arg(short = 'O', default_value = "3")]
    уровень_оптимизации: u8,

    /// Целевой процессор (native, generic, znver3, skylake)
    #[arg(long = "целевой-процессор", default_value = "native")]
    целевой_процессор: String,

}

#[derive(Subcommand, Debug)]
enum Команда {
    /// Скомпилировать файл (R7)
    #[command(name = "собрать", alias = "build", alias = "b")]
    Собрать {
        входной_файл: PathBuf,
        #[arg(short = 'o', long = "выход")]
        выходной_файл: Option<PathBuf>,
        #[arg(long)]
        llvm: bool,
    },

    /// Скомпилировать и запустить (R7 — cargo-script)
    #[command(name = "запустить", alias = "run", alias = "r")]
    Запустить {
        входной_файл: PathBuf,
        #[arg(long)]
        llvm: bool,
        /// Аргументы для программы
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        аргументы: Vec<String>,
    },

    /// Запустить автогенерированные тесты (Идея 20)
    #[command(name = "тест", alias = "test", alias = "t")]
    Тест {
        входной_файл: PathBuf,
        #[arg(long)]
        llvm: bool,
    },

    /// Показать версию
    #[command(name = "версия", alias = "version")]
    Версия,
}

fn main() -> Result<()> {
    let аргументы = Аргументы::parse();

    match аргументы.команда {
        Some(Команда::Версия) => {
            println!("zipc {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(Команда::Запустить {
            входной_файл,
            llvm,
            аргументы: арг,
        }) => {
            return команда_запустить(&входной_файл, llvm, &арг);
        }
        Some(Команда::Тест {
            входной_файл,
            llvm,
        }) => {
            return команда_тест(&входной_файл, llvm);
        }
        Some(Команда::Собрать {
            входной_файл,
            выходной_файл,
            llvm,
        }) => {
            let а = Аргументы {
                команда: None,
                входной_файл: Some(входной_файл),
                выходной_файл: выходной_файл.unwrap_or(аргументы.выходной_файл),
                llvm: llvm || аргументы.llvm,
                только_лексика: аргументы.только_лексика,
                только_парсинг: аргументы.только_парсинг,
                вывод_пп: аргументы.вывод_пп,
                вывод_ассемблера: аргументы.вывод_ассемблера,
                вывод_llvm: аргументы.вывод_llvm,
                байткод: аргументы.байткод,
                без_владения: аргументы.без_владения,
                запустить_тесты: аргументы.запустить_тесты,
                уровень_оптимизации: аргументы.уровень_оптимизации,
                целевой_процессор: аргументы.целевой_процессор,
            };
            return выполнить_сборку(а);
        }
        None => {}
    }

    if аргументы.входной_файл.is_none() {
        bail!("Укажите входной файл или подкоманду (собрать / запустить / тест)");
    }

    выполнить_сборку(аргументы)
}

/// R7: compile + run во временный каталог.
fn команда_запустить(входной_файл: &Path, llvm: bool, аргументы: &[String]) -> Result<()> {
    let временный = std::env::temp_dir().join(format!("zip_run_{}", std::process::id()));
    println!("Компиляция {}...", входной_файл.display());

    let zipc = std::env::current_exe().context("current_exe")?;
    let mut cmd = Command::new(&zipc);
    cmd.arg(входной_файл).arg("-o").arg(&временный);
    if llvm {
        cmd.arg("--llvm");
    }
    let статус = cmd
        .status()
        .with_context(|| format!("запуск {}", zipc.display()))?;
    if !статус.success() {
        bail!("Ошибка компиляции");
    }

    let путь_exe = if cfg!(windows) {
        with_extension(&временный, "exe")
    } else {
        временный.clone()
    };
    if !путь_exe.exists() {
        bail!("Исполняемый файл не создан: {}", путь_exe.display());
    }

    println!("Запуск {}\n", путь_exe.display());
    let статус = Command::new(&путь_exe)
        .args(аргументы)
        .status()
        .with_context(|| format!("запуск {}", путь_exe.display()))?;

    let _ = fs::remove_file(&путь_exe);
    let _ = fs::remove_file(with_extension(&временный, "ll"));
    let _ = fs::remove_file(with_extension(&временный, "o"));
    let _ = fs::remove_file(with_extension(&временный, "s"));

    if !статус.success() {
        std::process::exit(статус.code().unwrap_or(1));
    }
    Ok(())
}

/// Идея 20: сгенерировать тесты из `@тест`, скомпилировать и запустить.
fn команда_тест(входной_файл: &Path, llvm: bool) -> Result<()> {
    let исходник = fs::read_to_string(входной_файл).with_context(|| {
        format!("Не удалось прочитать «{}»", входной_файл.display())
    })?;

    let mut парсер =
        парсер::Парсер::новый(&исходник).map_err(|е| anyhow::anyhow!("{}", е))?;
    let дерево = парсер.разобрать().map_err(|е| anyhow::anyhow!("{}", е))?;
    let mut генератор_пп = промежуток::ГенераторПП::новый();
    let функции_пп = генератор_пп
        .сгенерировать(&дерево)
        .map_err(|е| anyhow::anyhow!("{}", е))?;

    let функции_с_тестами: Vec<_> = функции_пп
        .iter()
        .filter(|ф| ф.тест_случай.is_some() && ф.имя != "главная")
        .collect();
    if функции_с_тестами.is_empty() {
        println!("Нет функций с аннотацией @тест");
        return Ok(());
    }

    println!("=== ЗАПУСК АВТОГЕНЕРИРОВАННЫХ ТЕСТОВ ===");
    println!("Функций: {}", функции_с_тестами.len());

    let mut генератор = генератор_тестов::ГенераторТестов::новый();
    let тесты = генератор.сгенерировать_все(&функции_пп);
    println!("Сгенерировано тестов: {}", тесты.len());

    let stem = входной_файл
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("тесты");
    let путь_тестов = входной_файл
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}_автотесты.zip", stem));
    let файл_тестов = генератор_тестов::собрать_исходник_тестов(&исходник, &тесты);
    fs::write(&путь_тестов, файл_тестов)
        .with_context(|| format!("запись {}", путь_тестов.display()))?;
    println!("Файл тестов: {}", путь_тестов.display());

    let выход = входной_файл
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}_автотесты", stem));

    let zipc = std::env::current_exe().context("current_exe")?;
    let mut cmd = Command::new(&zipc);
    cmd.arg(&путь_тестов).arg("-o").arg(&выход);
    if llvm {
        cmd.arg("--llvm");
    }
    let статус = cmd.status().context("компиляция автотестов")?;
    if !статус.success() {
        bail!("Ошибка компиляции автотестов");
    }

    let exe = if cfg!(windows) {
        with_extension(&выход, "exe")
    } else {
        выход.clone()
    };
    let статус_запуска = Command::new(&exe)
        .status()
        .with_context(|| format!("запуск {}", exe.display()))?;
    if статус_запуска.success() {
        println!("\n✓ Все тесты прошли");
    } else {
        println!("\n✗ Некоторые тесты упали");
        std::process::exit(статус_запуска.code().unwrap_or(1));
    }
    Ok(())
}

fn выполнить_сборку(аргументы: Аргументы) -> Result<()> {
    let входной_файл = аргументы
        .входной_файл
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Не указан входной файл"))?;

    if аргументы.уровень_оптимизации > 3 {
        bail!("Уровень оптимизации должен быть в диапазоне 0–3");
    }

    if аргументы.запустить_тесты {
        return команда_тест(&входной_файл, аргументы.llvm);
    }

    // 1. Читаем исходник
    let исходник = fs::read_to_string(&входной_файл).with_context(|| {
        format!(
            "Не удалось прочитать файл «{}»",
            входной_файл.display()
        )
    })?;

    println!("=== КОМПИЛЯТОР ZIP v0.1.0 ===");
    println!("Файл: {}", входной_файл.display());
    println!("Размер: {} байт", исходник.len());
    println!(
        "Оптимизация: -O{}, целевой процессор: {}",
        аргументы.уровень_оптимизации, аргументы.целевой_процессор
    );

    // 2. Лексический анализ
    let mut лексер = лексика::Лексер::новый(&исходник);
    let токены = лексер
        .разобрать()
        .map_err(|е| anyhow::anyhow!("{}", е))?;
    println!("Токенов: {}", токены.len());

    if аргументы.только_лексика {
        for т in &токены {
            println!("  [{}:{}] {}", т.строка, т.столбец, т.токен);
        }
        return Ok(());
    }

    // 3. Парсинг
    let mut парсер = парсер::Парсер::новый(&исходник).map_err(|е| anyhow::anyhow!("{}", е))?;
    let дерево = парсер.разобрать().map_err(|е| anyhow::anyhow!("{}", е))?;
    let узлов = посчитать_узлы(&дерево);
    println!("Узлов AST: {}", узлов);

    if аргументы.только_парсинг {
        println!("{:#?}", дерево);
        return Ok(());
    }

    // 3b. #runtime("...") → clang -O3 по C-зеркалу STD (пока нет полного lowering массивов)
    if let Some(rt) = найти_runtime(&дерево) {
        let exe = скомпилировать_runtime(&rt, &аргументы.выходной_файл)?;
        println!("✓ Runtime «{}» → {}", rt, exe.display());
        println!("  (clang -O3 -march=native, зеркало ZIP STD)");
        return Ok(());
    }

    // 4. Генерация ПП
    let mut генератор_пп = промежуток::ГенераторПП::новый();
    let mut функции_пп = генератор_пп
        .сгенерировать(&дерево)
        .map_err(|е| anyhow::anyhow!("{}", е))?;
    println!("Функций ПП: {}", функции_пп.len());


    // 4epi. Идея 2 — лог применённых аннотаций + автогенерация @тест
    {
        let с_флагами: Vec<_> = функции_пп
            .iter()
            .filter(|ф| !аннотации::описание_флагов(ф).is_empty())
            .collect();
        if !с_флагами.is_empty() {
            println!("Применение аннотаций:");
            for ф in &с_флагами {
                println!("  {}: {}", ф.имя, аннотации::описание_флагов(ф).join(", "));
            }
        }
        let с_тестами: Vec<_> = функции_пп
            .iter()
            .filter(|ф| ф.тест_случай.is_some() && ф.имя != "главная")
            .collect();
        if !с_тестами.is_empty() {
            let mut генератор = генератор_тестов::ГенераторТестов::новый();
            let тесты = генератор.сгенерировать_все(&функции_пп);
            let файл_тестов = генератор_тестов::собрать_исходник_тестов(&исходник, &тесты);
            let путь_тестов = format!("{}_тесты.zip", аргументы.выходной_файл.display());
            fs::write(&путь_тестов, &файл_тестов)
                .with_context(|| format!("запись {}", путь_тестов))?;
            println!("  Тесты: {} ({} кейсов)", путь_тестов, тесты.len());
        }
    }

    // 4a. R2 — comptime: свернуть @comptime-вызовы в константы
    let есть_comptime = функции_пп.iter().any(|ф| ф.comptime);
    if есть_comptime {
        println!("Выполнение comptime-функций...");
        match компилятор_времени::свернуть_comptime_вызовы(&mut функции_пп) {
            Ok(журнал) => {
                if журнал.is_empty() {
                    println!("  (нет константных вызовов для свёртки)");
                }
                for с in &журнал {
                    let args = с
                        .аргументы
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!(
                        "  {}({}) = {:?}",
                        с.имя, args, с.результат
                    );
                }
            }
            Err(е) => {
                bail!("Ошибка comptime: {}", е);
            }
        }
        println!("Функций ПП после comptime: {}", функции_пп.len());
    }

    if аргументы.вывод_пп {
        for функция in &функции_пп {
            println!(
                "--- Функция: {} (async={}, perfect={}, comptime={}) ---",
                функция.имя, функция.асинхронная, функция.совершенная, функция.comptime
            );
            for инструкция in &функция.тело {
                println!("  {:?}", инструкция);
            }
        }
        return Ok(());
    }

    // 4b. R1 — анализ владения импульсами (Polonius-style)
    if !аргументы.без_владения {
        let ошибки = владение::проанализировать_функции(&функции_пп);
        if !ошибки.is_empty() {
            eprint!("{}", владение::форматировать_ошибки(&ошибки));
            std::process::exit(1);
        }
        println!("Владение: ✓ ошибок нет");
    } else {
        println!("Владение: пропущено (--без-владения)");
    }

    let имя_модуля = входной_файл
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("zip_модуль")
        .to_string();

    // 5a. Байт-код .zipc для zip-vm
    if аргументы.байткод {
        let путь = with_extension(&аргументы.выходной_файл, "zipc");
        байткод_эмиттер::записать_zipc(&функции_пп, &путь)
            .map_err(|е| anyhow::anyhow!("{}", е))?;
        println!("✓ Байт-код: {}", путь.display());
        println!("  Запуск: zip-vm {}", путь.display());
        return Ok(());
    }

    // 5b. LLVM IR → clang -O3
    if аргументы.llvm || аргументы.вывод_llvm {
        let ir = llvm_ir::сгенерировать_llvm_ir(&функции_пп, &имя_модуля)
            .map_err(|е| anyhow::anyhow!("{}", е))?;
        let путь_ll = with_extension(&аргументы.выходной_файл, "ll");
        fs::write(&путь_ll, &ir).with_context(|| format!("запись {}", путь_ll.display()))?;
        println!("LLVM IR: {}", путь_ll.display());
        println!("  Passes: {}", llvm_ir::LLVM_PASSES.join(", "));

        if аргументы.вывод_llvm {
            return Ok(());
        }

        match llvm_ir::скомпилировать_llvm(&функции_пп, &имя_модуля, &аргументы.выходной_файл) {
            Ok((exe, ll)) => {
                println!("✓ LLVM+clang -O3: {}", exe.display());
                println!("  IR: {}", ll.display());
            }
            Err(е) => {
                eprintln!("✗ LLVM-сборка не удалась: {}", е);
            }
        }
        return Ok(());
    }

    // 5b. ASM-бэкенд (x86-64)
    let mut опции = нативный::ОпцииНативнойСборки::default();
    опции.opt_level = аргументы.уровень_оптимизации;
    опции.target_cpu_native = аргументы.целевой_процессор == "native";
    опции.имя_модуля = имя_модуля;

    let mut генератор = нативный::ГенераторНативногоКода::с_опциями(опции);
    генератор
        .сгенерировать(&функции_пп)
        .map_err(|е| anyhow::anyhow!("Ошибка генерации нативного кода: {}", е))?;

    let путь_асм = with_extension(&аргументы.выходной_файл, "s");
    генератор
        .записать_ассемблер(путь_асм.to_str().unwrap())
        .map_err(|е| anyhow::anyhow!("Не удалось записать ассемблер: {}", е))?;
    println!("Ассемблер: {}", путь_асм.display());

    if аргументы.вывод_ассемблера {
        return Ok(());
    }

    let путь_объекта = with_extension(&аргументы.выходной_файл, "o");
    генератор
        .записать_объектный_файл(путь_объекта.to_str().unwrap())
        .map_err(|е| anyhow::anyhow!("Не удалось записать объектный файл: {}", е))?;
    println!("Объектный файл: {}", путь_объекта.display());

    // 6. Линковка (clang → gcc → cl → cc)
    let путь_исполняемого = аргументы.выходной_файл.clone();
    match нативный::линковать(&путь_асм, &путь_исполняемого) {
        Ok(путь) => {
            println!("✓ Исполняемый файл: {}", путь.display());
            if let Some(линкер) = нативный::найти_линкер() {
                println!("  Линкер: {}", линкер);
            }
        }
        Err(е) => {
            eprintln!("✗ Линковка пропущена или не удалась: {}", е);
            eprintln!(
                "  Ассемблер готов: соберите вручную, например: clang {} -o {}",
                путь_асм.display(),
                путь_исполняемого.display()
            );
        }
    }

    Ok(())
}

fn with_extension(path: &PathBuf, ext: &str) -> PathBuf {
    let mut путь = path.clone();
    путь.set_extension(ext);
    путь
}

fn найти_runtime(узел: &Узел) -> Option<String> {
    match узел {
        Узел::Runtime { имя } => Some(имя.clone()),
        Узел::Программа { объявления } => объявления.iter().find_map(найти_runtime),
        Узел::Блок(эл) => эл.iter().find_map(найти_runtime),
        _ => None,
    }
}

fn корень_репозитория() -> PathBuf {
    // zipc/ → ..
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn найти_openssl_префикс() -> Option<PathBuf> {
    let кандидаты = [
        PathBuf::from(r"C:\Program Files\FireDaemon OpenSSL 4"),
        PathBuf::from(r"C:\Program Files\OpenSSL-Win64"),
        PathBuf::from(r"C:\Program Files\OpenSSL"),
        PathBuf::from(r"C:\OpenSSL-Win64"),
        PathBuf::from("/usr"),
        PathBuf::from("/usr/local"),
        PathBuf::from("/opt/homebrew"),
    ];
    for п in кандидаты {
        if п.join("include/openssl/ssl.h").exists() {
            return Some(п);
        }
    }
    // vcpkg / env
    if let Ok(root) = std::env::var("OPENSSL_ROOT_DIR") {
        let п = PathBuf::from(root);
        if п.join("include/openssl/ssl.h").exists() {
            return Some(п);
        }
    }
    None
}

fn скомпилировать_runtime(имя: &str, выход: &PathBuf) -> Result<PathBuf> {
    use std::process::Command;

    let корень = корень_репозитория();
    let (источники, макрос, openssl, winsock): (Vec<PathBuf>, &str, bool, bool) = match имя {
        "bench_gemm" => (
            vec![корень.join("zip-std/runtime/gemm.c")],
            "ZIP_BENCH_MAIN",
            false,
            false,
        ),
        "train_xor" => (
            vec![корень.join("zip-std/runtime/nn_xor.c")],
            "ZIP_XOR_MAIN",
            false,
            false,
        ),
        "lexer_zip" => (
            vec![корень.join("zip-self/лексика_rt.c")],
            "ZIP_LEXER_MAIN",
            false,
            false,
        ),
        "parser_zip" | "parser_rt" | "parser_ast" => (
            vec![корень.join("zip-self/парсер_rt.c")],
            "ZIP_PARSER_MAIN",
            false,
            false,
        ),
        "ir_zip" | "ir_ast" | "промежуток" | "промежуток_rt" => (
            vec![корень.join("zip-self/промежуток_rt.c")],
            "ZIP_IR_MAIN",
            false,
            false,
        ),
        "native_rt" | "нативный" | "нативный_rt" => (
            vec![корень.join("zip-self/нативный_rt.c")],
            "ZIP_NATIVE_MAIN",
            false,
            false,
        ),
        "http_client" => (
            vec![корень.join("zip-std/runtime/http_client.c")],
            "ZIP_HTTP_MAIN",
            false,
            true,
        ),
        "https_client" => (
            vec![
                корень.join("zip-std/runtime/http_client.c"),
                корень.join("zip-std/runtime/https_client.c"),
            ],
            "ZIP_HTTPS_MAIN",
            true,
            true,
        ),
        "crypto" => (
            vec![корень.join("zip-std/runtime/crypto.c")],
            "ZIP_CRYPTO_MAIN",
            true,
            false,
        ),
        "iter" => (
            vec![корень.join("zip-std/runtime/iter.c")],
            "ZIP_ITER_MAIN",
            false,
            false,
        ),
        "undo" => (
            vec![корень.join("zip-std/runtime/undo.c")],
            "ZIP_UNDO_MAIN",
            false,
            false,
        ),
        "cache" => (
            vec![корень.join("zip-std/runtime/cache.c")],
            "ZIP_CACHE_MAIN",
            false,
            false,
        ),
        "parallel" => (
            vec![корень.join("zip-std/runtime/parallel.c")],
            "ZIP_PARALLEL_MAIN",
            false,
            false,
        ),
        "union" => (
            vec![корень.join("zip-std/runtime/union.c")],
            "ZIP_UNION_MAIN",
            false,
            false,
        ),
        "ttl" => (
            vec![корень.join("zip-std/runtime/ttl.c")],
            "ZIP_TTL_MAIN",
            false,
            false,
        ),
        "units" => (
            vec![корень.join("zip-std/runtime/units.c")],
            "ZIP_UNITS_MAIN",
            false,
            false,
        ),
        "stream" => (
            vec![корень.join("zip-std/runtime/stream.c")],
            "ZIP_STREAM_MAIN",
            false,
            false,
        ),
        "agent" => (
            vec![корень.join("zip-std/runtime/agent.c")],
            "ZIP_AGENT_MAIN",
            false,
            false,
        ),
        другой => bail!("неизвестный #runtime(\"{}\")", другой),
    };

    for источник in &источники {
        if !источник.exists() {
            bail!("файл runtime не найден: {}", источник.display());
        }
    }

    let mut exe = выход.clone();
    if cfg!(windows) && exe.extension().is_none() {
        exe.set_extension("exe");
    }

    let mut cmd = Command::new("clang");
    for источник in &источники {
        cmd.arg(источник);
    }
    cmd.arg("-O3")
        .arg("-march=native")
        .arg(format!("-D{}", макрос))
        .arg("-o")
        .arg(&exe);
    // Deterministic PE on Windows (stable self-host hashes)
    if cfg!(windows) {
        cmd.arg("-Wl,/Brepro");
    }
    // Avoid duplicate main when http_client.c is linked with https_client.c
    if имя == "https_client" {
        cmd.arg("-DZIP_HTTP_MAIN_DISABLED");
    }

    if openssl {
        let префикс = найти_openssl_префикс().ok_or_else(|| {
            anyhow::anyhow!(
                "OpenSSL не найден (ожидался include/openssl/ssl.h). Установите ShiningLight.OpenSSL.Dev"
            )
        })?;
        cmd.arg(format!("-I{}/include", префикс.display()));
        let lib = префикс.join("lib");
        if lib.exists() {
            cmd.arg(format!("-L{}", lib.display()));
        }
        let vc_lib = префикс.join("lib/VC/x64/MD");
        if vc_lib.exists() {
            cmd.arg(format!("-L{}", vc_lib.display()));
        }
        // Windows: libssl.lib / libcrypto.lib; Unix: -lssl -lcrypto
        if cfg!(windows) {
            let ssl = lib.join("libssl.lib");
            let crypto = lib.join("libcrypto.lib");
            if ssl.exists() && crypto.exists() {
                cmd.arg(ssl).arg(crypto);
            } else {
                cmd.arg("-llibssl").arg("-llibcrypto");
            }
        } else {
            cmd.arg("-lssl").arg("-lcrypto");
        }
        println!("  OpenSSL: {}", префикс.display());
    }

    if !cfg!(windows) {
        cmd.arg("-lm");
    }
    if winsock && cfg!(windows) {
        cmd.arg("-lws2_32");
        cmd.arg("-lcrypt32");
    }
    if (имя == "parallel" || имя == "agent") && !cfg!(windows) {
        cmd.arg("-lpthread");
    }
    // clock_gettime на старых Linux — librt
    if имя == "ttl" && !cfg!(windows) {
        cmd.arg("-lrt");
    }

    let статус = cmd
        .status()
        .with_context(|| "запуск clang для #runtime")?;
    if !статус.success() {
        bail!("clang не смог собрать runtime «{}»", имя);
    }
    Ok(exe)
}

fn посчитать_узлы(узел: &Узел) -> usize {
    match узел {
        Узел::Программа { объявления } => {
            1 + объявления.iter().map(посчитать_узлы).sum::<usize>()
        }
        Узел::ОбъявлениеКорня { .. } => 1,
        Узел::ВызовГлагола {
            аргументы,
            следующий,
            ..
        } => {
            1 + аргументы.iter().map(посчитать_узлы).sum::<usize>()
                + следующий.as_ref().map(|u| посчитать_узлы(u)).unwrap_or(0)
        }
        Узел::Фиксация { данные, .. } => 1 + посчитать_узлы(данные),
        Узел::АсинхронныйПоток { тело, .. } => 1 + посчитать_узлы(тело),
        Узел::Хор { ветки } => 1 + ветки.iter().map(посчитать_узлы).sum::<usize>(),
        Узел::Блок(эл) => 1 + эл.iter().map(посчитать_узлы).sum::<usize>(),
        Узел::Если {
            условие,
            тогда,
            иначе,
        } => {
            1 + посчитать_узлы(условие)
                + посчитать_узлы(тогда)
                + иначе.as_ref().map(|u| посчитать_узлы(u)).unwrap_or(0)
        }
        Узел::Пока { условие, тело } => 1 + посчитать_узлы(условие) + посчитать_узлы(тело),
        Узел::Для {
            коллекция,
            тело,
            ..
        } => 1 + посчитать_узлы(коллекция) + посчитать_узлы(тело),
        Узел::ДляИтератор {
            итератор,
            тело,
            ..
        } => 1 + посчитать_узлы(итератор) + посчитать_узлы(тело),
        Узел::Диапазон { начало, конец } => 1 + посчитать_узлы(начало) + посчитать_узлы(конец),
        Узел::Возврат(Some(з)) => 1 + посчитать_узлы(з),
        Узел::Возврат(None) | Узел::Прервать | Узел::Продолжить => 1,
        Узел::ОпределениеФункции { тело, .. } => 1 + посчитать_узлы(тело),
        Узел::СпециализацияГлагола { варианты, .. } => {
            1 + варианты
                .iter()
                .map(|в| посчитать_узлы(&в.тело))
                .sum::<usize>()
        }
        Узел::ПотокДанных {
            источник,
            преобразования,
            приёмник,
            ..
        } => {
            1 + посчитать_узлы(источник)
                + преобразования
                    .iter()
                    .map(|(_, у)| посчитать_узлы(у))
                    .sum::<usize>()
                + посчитать_узлы(приёмник)
        }
        Узел::Агент {
            состояние,
            методы,
            ..
        } => {
            1 + состояние
                .iter()
                .map(|(_, _, з)| посчитать_узлы(з))
                .sum::<usize>()
                + методы.iter().map(посчитать_узлы).sum::<usize>()
        }
        Узел::Реагировать { событие, тело } => {
            1 + посчитать_узлы(событие)
                + тело.as_ref().map(|u| посчитать_узлы(u)).unwrap_or(0)
        }
        Узел::Отправить { сообщение } => 1 + посчитать_узлы(сообщение),
        Узел::ДоступКПолю { объект, .. } => 1 + посчитать_узлы(объект),
        Узел::ВызовМетода {
            объект,
            аргументы,
            ..
        } => 1 + посчитать_узлы(объект) + аргументы.iter().map(посчитать_узлы).sum::<usize>(),
        Узел::СвязатьБиблиотеку { .. } | Узел::Runtime { .. } => 1,
        Узел::ВнешнийВызов { аргументы, .. } => {
            1 + аргументы.iter().map(посчитать_узлы).sum::<usize>()
        }
        Узел::ВызовФункции { аргументы, .. } => {
            1 + аргументы.iter().map(посчитать_узлы).sum::<usize>()
        }
        Узел::ЛогическоеИ(а, б)
        | Узел::ЛогическоеИли(а, б)
        | Узел::Сравнение {
            левый: а,
            правый: б,
            ..
        }
        | Узел::БинарнаяОперация {
            левый: а,
            правый: б,
            ..
        } => 1 + посчитать_узлы(а) + посчитать_узлы(б),
        Узел::ЛогическоеНе(а) | Узел::УнарныйМинус(а) | Узел::Присваивание { значение: а, .. } => {
            1 + посчитать_узлы(а)
        }
        Узел::Истина | Узел::Ложь | Узел::Ничто => 1,
        Узел::Нечто { значение }
        | Узел::Успех { значение }
        | Узел::Ошибка { значение } => 1 + посчитать_узлы(значение),
        Узел::ПроверкаТипа { выражение, .. } => 1 + посчитать_узлы(выражение),
        Узел::Целое(_)
        | Узел::Вещественное(_)
        | Узел::Строка(_)
        | Узел::Идентификатор(_) => 1,
    }
}

