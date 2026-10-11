//! Поиск zipc / selfc, запуск сборки и HTTP-реестр.

use anyhow::{bail, Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::{Archive, Builder};

pub fn корень_репозитория() -> PathBuf {
    if let Ok(r) = std::env::var("ZIP_ROOT") {
        return PathBuf::from(r);
    }
    // зипм/ → ..
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn найти_компилятор() -> Result<PathBuf> {
    let root = корень_репозитория();
    let кандидаты = [
        root.join("zipc/target/release/zipc.exe"),
        root.join("zipc/target/release/zipc"),
        root.join("zip-self/selfc_v3.exe"),
        root.join("zip-self/selfc.exe"),
        root.join("zip-self/selfc_v1.exe"),
        PathBuf::from("zipc.exe"),
        PathBuf::from("zipc"),
    ];
    for п in &кандидаты {
        if п.exists() {
            return Ok(п.clone());
        }
    }
    // PATH
    if which("zipc").is_some() {
        return Ok(PathBuf::from("zipc"));
    }
    bail!(
        "не найден zipc/selfc. Соберите: cargo build --release --manifest-path zipc/Cargo.toml\n\
         или задайте ZIP_ROOT={}",
        root.display()
    )
}

fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        for dir in std::env::split_paths(&paths) {
            for candidate in [
                dir.join(name),
                dir.join(format!("{name}.exe")),
            ] {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
        None
    })
}

pub fn скомпилировать(исходник: &Path, выход: &Path, llvm: bool) -> Result<()> {
    let компилятор = найти_компилятор()?;
    if let Some(род) = выход.parent() {
        std::fs::create_dir_all(род)?;
    }

    let mut cmd = Command::new(&компилятор);
    cmd.arg(исходник).arg("-o").arg(выход);
    if llvm {
        cmd.arg("--llvm");
    }

    println!("  {} {} -o {}{}",
        компилятор.display(),
        исходник.display(),
        выход.display(),
        if llvm { " --llvm" } else { "" }
    );

    let статус = cmd
        .status()
        .with_context(|| format!("запуск {}", компилятор.display()))?;
    if !статус.success() {
        bail!("компилятор завершился с кодом {:?}", статус.code());
    }
    Ok(())
}

pub fn путь_exe(выход: &Path) -> PathBuf {
    if cfg!(windows) && выход.extension().is_none() {
        выход.with_extension("exe")
    } else {
        выход.to_path_buf()
    }
}

/// Базовый URL HTTP-реестра (если ZIPM_REGISTRY начинается с http).
pub fn url_реестра() -> Option<String> {
    let v = std::env::var("ZIPM_REGISTRY")
        .or_else(|_| std::env::var("ЗИПМ_РЕЕСТР"))
        .ok()?;
    let t = v.trim();
    if t.starts_with("http://") || t.starts_with("https://") {
        Some(t.trim_end_matches('/').to_string())
    } else {
        None
    }
}

fn токен_авторизации() -> Option<String> {
    std::env::var("ZIPM_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .ok()
}

/// Упаковать каталог пакета в .tar.gz.
pub fn создать_архив(путь_пакета: &Path) -> Result<Vec<u8>> {
    let буфер = Cursor::new(Vec::new());
    let энкодер = GzEncoder::new(буфер, Compression::default());
    let mut tar = Builder::new(энкодер);
    tar.append_dir_all(".", путь_пакета)
        .with_context(|| format!("архивация {}", путь_пакета.display()))?;
    let энкодер = tar.into_inner()?;
    let курсор = энкодер.finish()?;
    Ok(курсор.into_inner())
}

/// Распаковать .tar.gz в каталог.
pub fn распаковать_архив(байты: &[u8], назначение: &Path) -> Result<()> {
    std::fs::create_dir_all(назначение)?;
    let декодер = flate2::read::GzDecoder::new(Cursor::new(байты));
    let mut архив = Archive::new(декодер);
    архив
        .unpack(назначение)
        .with_context(|| format!("распаковка в {}", назначение.display()))?;
    Ok(())
}

/// POST /пакет/{имя}/{версия} в HTTP-реестр.
pub fn опубликовать_в_реестр(имя: &str, версия: &str, путь_пакета: &Path) -> Result<()> {
    let база = url_реестра().unwrap_or_else(|| "https://registry.zip-lang.org".into());
    let url = format!("{база}/пакет/{имя}/{версия}");
    let tar_gz = создать_архив(путь_пакета)?;

    let mut req = ureq::post(&url)
        .set("Content-Type", "application/gzip")
        .set("User-Agent", "zipm/0.1");
    if let Some(t) = токен_авторизации() {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }

    let ответ = req
        .send_bytes(&tar_gz)
        .with_context(|| format!("POST {url}"))?;
    let status = ответ.status();
    if !(200..300).contains(&status) {
        let mut тело = String::new();
        let _ = ответ.into_reader().read_to_string(&mut тело);
        bail!("Ошибка публикации HTTP {status}: {тело}");
    }
    println!("✓ Пакет {имя} {версия} опубликован → {url}");
    Ok(())
}

/// GET /пакет/{имя}/{версия} из HTTP-реестра.
pub fn установить_из_реестра(имя: &str, версия: &str, назначение: &Path) -> Result<()> {
    let база = url_реестра().unwrap_or_else(|| "https://registry.zip-lang.org".into());
    let url = format!("{база}/пакет/{имя}/{версия}");
    let ответ = ureq::get(&url)
        .set("User-Agent", "zipm/0.1")
        .call()
        .with_context(|| format!("GET {url}"))?;
    if ответ.status() != 200 {
        bail!("Пакет не найден: {имя} {версия} (HTTP {})", ответ.status());
    }
    let mut байты = Vec::new();
    ответ
        .into_reader()
        .read_to_end(&mut байты)
        .context("чтение тела ответа")?;

    // gzip magic или plain манифест
    if байты.starts_with(&[0x1f, 0x8b]) {
        распаковать_архив(&байты, назначение)?;
    } else {
        std::fs::create_dir_all(назначение)?;
        std::fs::write(назначение.join("зипм.манифест"), &байты)?;
    }
    println!("✓ Пакет {имя} {версия} установлен → {}", назначение.display());
    Ok(())
}

/// Записать файл (helper для тестов / локальной публикации).
pub fn записать_файл(путь: &Path, данные: &[u8]) -> Result<()> {
    if let Some(p) = путь.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut f = File::create(путь)?;
    f.write_all(данные)?;
    Ok(())
}
