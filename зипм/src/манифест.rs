//! Разбор и запись `зипм.манифест`.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const ИМЯ_МАНИФЕСТА: &str = "зипм.манифест";
pub const ИМЯ_МАНИФЕСТА_ASCII: &str = "zipm.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Манифест {
    pub имя: String,
    #[serde(default = "версия_по_умолчанию")]
    pub версия: String,
    #[serde(default)]
    pub автор: String,
    #[serde(default = "лицензия_по_умолчанию")]
    pub лицензия: String,
    #[serde(default)]
    pub описание: String,
    #[serde(default)]
    pub зависимости: BTreeMap<String, String>,
    #[serde(default, rename = "зависимости_разработки")]
    pub зависимости_разработки: BTreeMap<String, String>,
    #[serde(default)]
    pub сборка: Сборка,
    #[serde(default)]
    pub тесты: Тесты,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Сборка {
    #[serde(default = "тип_программы")]
    pub тип: String,
    #[serde(default = "точка_входа_по_умолчанию", rename = "точка_входа")]
    pub точка_входа: String,
    #[serde(default = "выход_по_умолчанию")]
    pub выход: String,
}

impl Default for Сборка {
    fn default() -> Self {
        Self {
            тип: тип_программы(),
            точка_входа: точка_входа_по_умолчанию(),
            выход: выход_по_умолчанию(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Тесты {
    #[serde(default = "каталог_тестов")]
    pub каталог: String,
}

impl Default for Тесты {
    fn default() -> Self {
        Self {
            каталог: каталог_тестов(),
        }
    }
}

fn версия_по_умолчанию() -> String {
    "0.1.0".into()
}
fn лицензия_по_умолчанию() -> String {
    "MIT".into()
}
fn тип_программы() -> String {
    "программа".into()
}
fn точка_входа_по_умолчанию() -> String {
    "src/главная.zip".into()
}
fn выход_по_умолчанию() -> String {
    "цель/выпуск/программа".into()
}
fn каталог_тестов() -> String {
    "tests/".into()
}

pub fn путь_манифеста(корень: &Path) -> PathBuf {
    let p = корень.join(ИМЯ_МАНИФЕСТА);
    if p.exists() {
        return p;
    }
    корень.join(ИМЯ_МАНИФЕСТА_ASCII)
}

pub fn прочитать(корень: &Path) -> Result<Манифест> {
    let путь = путь_манифеста(корень);
    let текст = fs::read_to_string(&путь)
        .with_context(|| format!("не удалось прочитать {}", путь.display()))?;
    toml::from_str(&текст).with_context(|| format!("некорректный манифест {}", путь.display()))
}

pub fn записать(корень: &Path, м: &Манифест) -> Result<()> {
    let текст = toml::to_string_pretty(м)?;
    let путь = корень.join(ИМЯ_МАНИФЕСТА);
    fs::write(&путь, текст)?;
    // ASCII-копия для инструментов без кириллицы
    let _ = fs::write(корень.join(ИМЯ_МАНИФЕСТА_ASCII), toml::to_string_pretty(м)?);
    Ok(())
}

pub fn найти_корень_пакета() -> Result<PathBuf> {
    let mut тек = std::env::current_dir()?;
    loop {
        if путь_манифеста(&тек).exists() {
            return Ok(тек);
        }
        if !тек.pop() {
            bail!("не найден {} (запустите из каталога пакета)", ИМЯ_МАНИФЕСТА);
        }
    }
}
