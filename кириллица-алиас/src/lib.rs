//! Утилита для создания кириллических алиасов бинарников.
//!
//! `зип-профиль` → hard link / symlink на `zip-profile`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Пары (ASCII-имя без расширения, кириллическое имя).
pub const АЛИАСЫ: &[(&str, &str)] = &[
    ("zip-profile", "зип-профиль"),
    ("zip-debugger", "зип-отладчик"),
    ("zip-lsp", "зип-lsp"),
    ("zipm", "зипм"),
];

fn с_расширением(dir: &Path, имя: &str) -> PathBuf {
    if cfg!(windows) {
        dir.join(format!("{имя}.exe"))
    } else {
        dir.join(имя)
    }
}

/// Создаёт hard link (Windows) или symlink (Unix) `кириллическое` → `ascii`.
pub fn создать_алиас(каталог: &Path, ascii_имя: &str, кириллическое: &str) -> io::Result<PathBuf> {
    let ascii_path = с_расширением(каталог, ascii_имя);
    let кириллица_path = с_расширением(каталог, кириллическое);

    if !ascii_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("не найден {}", ascii_path.display()),
        ));
    }

    if кириллица_path.exists() {
        // Уже есть — OK (возможно предыдущий hard link)
        return Ok(кириллица_path);
    }

    #[cfg(windows)]
    {
        fs::hard_link(&ascii_path, &кириллица_path)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&ascii_path, &кириллица_path)?;
    }

    #[cfg(not(any(windows, unix)))]
    {
        let _ = (&ascii_path, &кириллица_path);
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "платформа не поддерживает алиасы",
        ));
    }

    Ok(кириллица_path)
}

/// Создаёт все известные алиасы в `каталог` (обычно `target/release`).
pub fn создать_все_в(каталог: &Path) -> Vec<(String, Result<PathBuf, String>)> {
    АЛИАСЫ
        .iter()
        .map(|(ascii, кир)| {
            let r = создать_алиас(каталог, ascii, кир).map_err(|е| е.to_string());
            ((*кир).to_string(), r)
        })
        .collect()
}
