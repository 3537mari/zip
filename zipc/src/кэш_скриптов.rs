//! Кэш скомпилированных скриптов (R7).
//!
//! Если скрипт не изменился — используем готовый .exe из кэша.
//! Имя файла в кэше = SHA-256 содержимого исходника (переживает перезапуск процесса).

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Кэш скриптов.
pub struct КэшСкриптов {
    каталог: PathBuf,
}

impl КэшСкриптов {
    pub fn новый() -> Self {
        let каталог = std::env::temp_dir().join("zip-script-cache");
        let _ = fs::create_dir_all(&каталог);
        Self { каталог }
    }

    pub fn каталог(&self) -> &Path {
        &self.каталог
    }

    /// Вычислить хеш исходника (SHA-256).
    pub fn хеш_файла(путь: &Path) -> std::io::Result<String> {
        let содержимое = fs::read(путь)?;
        let mut hasher = Sha256::new();
        hasher.update(&содержимое);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Короткий хеш пути (для временных имён сборки).
    pub fn хеш_имени(путь: &Path) -> String {
        let mut hasher = Sha256::new();
        hasher.update(путь.to_string_lossy().as_bytes());
        format!("{:x}", hasher.finalize())[..8].to_string()
    }

    fn путь_для_хеша(&self, хеш: &str) -> PathBuf {
        if cfg!(windows) {
            self.каталог.join(format!("{}.exe", хеш))
        } else {
            self.каталог.join(хеш)
        }
    }

    /// Проверить, есть ли актуальная сборка.
    pub fn получить(&self, исходник: &Path) -> Option<PathBuf> {
        let хеш = Self::хеш_файла(исходник).ok()?;
        let путь = self.путь_для_хеша(&хеш);
        if путь.exists() {
            Some(путь)
        } else {
            None
        }
    }

    /// Сохранить скомпилированный бинарник в кэш.
    pub fn сохранить(&mut self, исходник: &Path, путь_exe: &Path) -> std::io::Result<()> {
        let хеш = Self::хеш_файла(исходник)?;
        let назначение = self.путь_для_хеша(&хеш);
        if путь_exe != назначение {
            fs::copy(путь_exe, &назначение)?;
        }
        let _ = SystemTime::now();
        let _ = назначение.metadata()?.len();
        Ok(())
    }

    /// Очистить кэш.
    pub fn очистить(&mut self) -> std::io::Result<()> {
        if self.каталог.exists() {
            for запись in fs::read_dir(&self.каталог)? {
                let запись = запись?;
                let путь = запись.path();
                if путь.is_file() {
                    let _ = fs::remove_file(путь);
                }
            }
        }
        Ok(())
    }

    /// Статистика: (число файлов, суммарный размер байт).
    pub fn статистика(&self) -> (usize, u64) {
        let mut всего = 0usize;
        let mut размер = 0u64;
        let Ok(чтение) = fs::read_dir(&self.каталог) else {
            return (0, 0);
        };
        for запись in чтение.flatten() {
            if let Ok(мета) = запись.metadata() {
                if мета.is_file() {
                    всего += 1;
                    размер += мета.len();
                }
            }
        }
        (всего, размер)
    }
}

impl Default for КэшСкриптов {
    fn default() -> Self {
        Self::новый()
    }
}
