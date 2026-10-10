//! Поиск zipc / selfc и запуск сборки.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

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
