//! Встроенные функции для VM v3 (+ runtime syscalls / аллокатор / I/O).

use crate::исполнитель_v3::ЗначениеV3;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Mutex;

pub type ВстроеннаяФункция = fn(&[ЗначениеV3]) -> Result<ЗначениеV3, String>;

struct Куча {
    данные: Vec<u8>,
    использовано: usize,
}

impl Куча {
    fn новая() -> Self {
        Self {
            данные: vec![0u8; 1024 * 1024],
            использовано: 0,
        }
    }

    fn выделить(&mut self, размер: usize) -> i64 {
        let n = (размер.max(1) + 15) & !15;
        if self.использовано + n > self.данные.len() {
            let доп = (16 * 1024 * 1024).max(n);
            self.данные.resize(self.данные.len() + доп, 0);
        }
        let addr = self.использовано as i64;
        // Храним «указатель» как смещение+1 (0 = null)
        let ptr = addr + 1;
        self.использовано += n;
        ptr
    }
}

static КУЧА: Mutex<Option<Куча>> = Mutex::new(None);
static ФАЙЛЫ: Mutex<Option<HashMap<i64, String>>> = Mutex::new(None);
static СЛЕД_FD: Mutex<i64> = Mutex::new(100);

fn куча_выделить(размер: i64) -> i64 {
    let mut g = КУЧА.lock().unwrap();
    if g.is_none() {
        *g = Some(Куча::новая());
    }
    g.as_mut().unwrap().выделить(размер.max(0) as usize)
}

fn куча_used() -> i64 {
    let g = КУЧА.lock().unwrap();
    g.as_ref().map(|k| k.использовано as i64).unwrap_or(0)
}

fn как_строка(з: &ЗначениеV3) -> String {
    match з {
        ЗначениеV3::Строка(s) => s.clone(),
        другой => другой.to_string(),
    }
}

fn sys_write_impl(fd: i64, данные: &[u8]) -> Result<i64, String> {
    match fd {
        1 => {
            let mut out = io::stdout().lock();
            out.write_all(данные).map_err(|e| e.to_string())?;
            out.flush().ok();
            Ok(данные.len() as i64)
        }
        2 => {
            let mut err = io::stderr().lock();
            err.write_all(данные).map_err(|e| e.to_string())?;
            err.flush().ok();
            Ok(данные.len() as i64)
        }
        _ => Err(format!("sys_write: fd {fd} не поддержан в VM")),
    }
}

fn zip_syscall_impl(арг: &[ЗначениеV3]) -> Result<ЗначениеV3, String> {
    let номер = арг.first().map(|з| з.как_целое()).unwrap_or(-1);
    let а1 = арг.get(1).map(|з| з.как_целое()).unwrap_or(0);
    let а2 = арг.get(2);
    let а3 = арг.get(3).map(|з| з.как_целое()).unwrap_or(0);
    match номер {
        1 => {
            // WRITE: fd, buffer(str|ptr), size
            let fd = а1;
            let bytes = match а2 {
                Some(ЗначениеV3::Строка(s)) => {
                    let n = if а3 > 0 {
                        (а3 as usize).min(s.len())
                    } else {
                        s.len()
                    };
                    s.as_bytes()[..n].to_vec()
                }
                Some(з) => {
                    let s = з.to_string();
                    let n = if а3 > 0 {
                        (а3 as usize).min(s.len())
                    } else {
                        s.len()
                    };
                    s.as_bytes()[..n].to_vec()
                }
                None => Vec::new(),
            };
            Ok(ЗначениеV3::Целое(sys_write_impl(fd, &bytes)?))
        }
        9 => {
            // MMAP: treat size as а2 or а1
            let размер = арг.get(2).map(|з| з.как_целое()).unwrap_or(а1);
            Ok(ЗначениеV3::Целое(куча_выделить(размер)))
        }
        60 | 231 => {
            // EXIT — в VM просто возвращаем код (не убиваем процесс тестов)
            Ok(ЗначениеV3::Целое(а1))
        }
        0 | 2 | 3 | 12 => Ok(ЗначениеV3::Целое(0)),
        _ => Err(format!("zip_syscall: номер {номер} не поддержан")),
    }
}

/// Реестр встроенных функций.
pub struct Встроенные {
    функции: HashMap<String, ВстроеннаяФункция>,
}

impl Встроенные {
    pub fn новый() -> Self {
        let mut ф = Self {
            функции: HashMap::new(),
        };

        ф.регистрация("печать_число", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            println!("{n}");
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("ПечатьЧисло", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            println!("{n}");
            Ok(ЗначениеV3::Ничто)
        });

        ф.регистрация("печать_строки", |арг| {
            let s = match арг.first() {
                Some(ЗначениеV3::Строка(s)) => s.clone(),
                Some(з) => format!("{з:?}"),
                None => String::new(),
            };
            println!("{s}");
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("ПечатьСтроки", |арг| {
            let s = match арг.first() {
                Some(ЗначениеV3::Строка(s)) => s.clone(),
                Some(з) => format!("{з:?}"),
                None => String::new(),
            };
            println!("{s}");
            Ok(ЗначениеV3::Ничто)
        });

        ф.регистрация("печать", |арг| {
            if let Some(з) = арг.first() {
                print!("{з}");
            }
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("Печать", |арг| {
            if let Some(з) = арг.first() {
                print!("{з}");
            }
            Ok(ЗначениеV3::Ничто)
        });

        ф.регистрация("длина_строки", |арг| {
            let s = match арг.first() {
                Some(ЗначениеV3::Строка(s)) => s.chars().count() as i64,
                _ => 0,
            };
            Ok(ЗначениеV3::Целое(s))
        });

        ф.регистрация("соединить_строки", |арг| {
            let a = match арг.first() {
                Some(ЗначениеV3::Строка(s)) => s.clone(),
                _ => String::new(),
            };
            let b = match арг.get(1) {
                Some(ЗначениеV3::Строка(s)) => s.clone(),
                _ => String::new(),
            };
            Ok(ЗначениеV3::Строка(format!("{a}{b}")))
        });

        ф.регистрация("модуль", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            Ok(ЗначениеV3::Целое(n.abs()))
        });
        ф.регистрация("минимум", |арг| {
            let a = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            let b = арг.get(1).map(|з| з.как_целое()).unwrap_or(0);
            Ok(ЗначениеV3::Целое(a.min(b)))
        });
        ф.регистрация("максимум", |арг| {
            let a = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            let b = арг.get(1).map(|з| з.как_целое()).unwrap_or(0);
            Ok(ЗначениеV3::Целое(a.max(b)))
        });

        ф.регистрация("сипра_создать", |арг| {
            let размер = арг.first().map(|з| з.как_целое()).unwrap_or(16);
            Ok(ЗначениеV3::Множество(Vec::with_capacity(размер.max(0) as usize)))
        });
        ф.регистрация("сипра_добавить", |арг| {
            match (арг.first(), арг.get(1)) {
                (Some(ЗначениеV3::Множество(м)), Some(значение)) => {
                    let mut новая = м.clone();
                    новая.push(значение.clone());
                    Ok(ЗначениеV3::Множество(новая))
                }
                _ => Err("Ожидалось множество".into()),
            }
        });
        ф.регистрация("сипра_длина", |арг| {
            let длина = match арг.first() {
                Some(ЗначениеV3::Множество(м)) => м.len() as i64,
                _ => 0,
            };
            Ok(ЗначениеV3::Целое(длина))
        });

        // === Runtime: syscalls ===
        ф.регистрация("zip_syscall", zip_syscall_impl);
        ф.регистрация("syscall", zip_syscall_impl);

        ф.регистрация("sys_write", |арг| {
            let fd = арг.first().map(|з| з.как_целое()).unwrap_or(1);
            let s = арг.get(1).map(как_строка).unwrap_or_default();
            let n = арг.get(2).map(|з| з.как_целое()).unwrap_or(s.len() as i64);
            let bytes = s.as_bytes();
            let take = (n.max(0) as usize).min(bytes.len());
            Ok(ЗначениеV3::Целое(sys_write_impl(fd, &bytes[..take])?))
        });
        ф.регистрация("sys_read", |_арг| Ok(ЗначениеV3::Целое(0)));
        ф.регистрация("sys_open", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let флаги = арг.get(1).map(|з| з.как_целое()).unwrap_or(0);
            let mut map = ФАЙЛЫ.lock().unwrap();
            if map.is_none() {
                *map = Some(HashMap::new());
            }
            let mut fd_g = СЛЕД_FD.lock().unwrap();
            let fd = *fd_g;
            *fd_g += 1;
            // 0=RDONLY placeholder, 1/flags with CREAT → empty file path registry
            let _ = флаги;
            map.as_mut().unwrap().insert(fd, путь);
            Ok(ЗначениеV3::Целое(fd))
        });
        ф.регистрация("sys_close", |_арг| Ok(ЗначениеV3::Целое(0)));
        ф.регистрация("sys_mmap", |арг| {
            let размер = арг.first().map(|з| з.как_целое()).unwrap_or(4096);
            Ok(ЗначениеV3::Целое(куча_выделить(размер)))
        });
        ф.регистрация("sys_exit", |арг| {
            let код = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            Ok(ЗначениеV3::Целое(код))
        });

        // === Runtime: allocator ===
        ф.регистрация("выделить", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            Ok(ЗначениеV3::Целое(куча_выделить(n)))
        });
        ф.регистрация("zip_alloc", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            Ok(ЗначениеV3::Целое(куча_выделить(n)))
        });
        ф.регистрация("освободить", |_арг| Ok(ЗначениеV3::Ничто));
        ф.регистрация("аллокатор_инициализировать", |_арг| {
            Ok(ЗначениеV3::Целое(куча_выделить(0)))
        });
        ф.регистрация("аллокатор_статистика", |_арг| Ok(ЗначениеV3::Целое(куча_used())));
        ф.регистрация("zip_alloc_used", |_арг| Ok(ЗначениеV3::Целое(куча_used())));

        // === Runtime: вывод ===
        ф.регистрация("напечатать", |арг| {
            let s = арг.first().map(как_строка).unwrap_or_default();
            let _ = sys_write_impl(1, s.as_bytes())?;
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("напечатать_строку", |арг| {
            let s = арг.first().map(как_строка).unwrap_or_default();
            let mut out = s.into_bytes();
            out.push(b'\n');
            let _ = sys_write_impl(1, &out)?;
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("напечатать_число", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            let s = n.to_string();
            let _ = sys_write_impl(1, s.as_bytes())?;
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("напечатать_число_строкой", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            let s = format!("{n}\n");
            let _ = sys_write_impl(1, s.as_bytes())?;
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("напечатать_ошибку", |арг| {
            let s = арг.first().map(как_строка).unwrap_or_default();
            let mut out = s.into_bytes();
            out.push(b'\n');
            let _ = sys_write_impl(2, &out)?;
            Ok(ЗначениеV3::Ничто)
        });
        ф.регистрация("zip_print_i64", |арг| {
            let n = арг.first().map(|з| з.как_целое()).unwrap_or(0);
            let s = n.to_string();
            let _ = sys_write_impl(1, s.as_bytes())?;
            Ok(ЗначениеV3::Ничто)
        });

        // === Runtime: файлы ===
        ф.регистрация("файл_открыть", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let режим = арг.get(1).map(как_строка).unwrap_or_else(|| "r".into());
            let mut map = ФАЙЛЫ.lock().unwrap();
            if map.is_none() {
                *map = Some(HashMap::new());
            }
            let mut fd_g = СЛЕД_FD.lock().unwrap();
            let fd = *fd_g;
            *fd_g += 1;
            map.as_mut()
                .unwrap()
                .insert(fd, format!("{путь}\0{режим}"));
            Ok(ЗначениеV3::Целое(fd))
        });
        ф.регистрация("файл_закрыть", |_арг| Ok(ЗначениеV3::Целое(0)));
        ф.регистрация("файл_записать", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let данные = арг.get(1).map(как_строка).unwrap_or_default();
            std::fs::write(&путь, данные.as_bytes()).map_err(|e| e.to_string())?;
            Ok(ЗначениеV3::Целое(данные.len() as i64))
        });
        ф.регистрация("файл_дописать", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let данные = арг.get(1).map(как_строка).unwrap_or_default();
            use std::fs::OpenOptions;
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&путь)
                .map_err(|e| e.to_string())?;
            f.write_all(данные.as_bytes()).map_err(|e| e.to_string())?;
            Ok(ЗначениеV3::Целое(данные.len() as i64))
        });
        ф.регистрация("файл_прочитать", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            match std::fs::read_to_string(&путь) {
                Ok(s) => Ok(ЗначениеV3::Строка(s)),
                Err(_) => Ok(ЗначениеV3::Целое(0)),
            }
        });
        ф.регистрация("zip_file_open", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let режим = арг.get(1).map(как_строка).unwrap_or_else(|| "r".into());
            let mut map = ФАЙЛЫ.lock().unwrap();
            if map.is_none() {
                *map = Some(HashMap::new());
            }
            let mut fd_g = СЛЕД_FD.lock().unwrap();
            let fd = *fd_g;
            *fd_g += 1;
            map.as_mut()
                .unwrap()
                .insert(fd, format!("{путь}\0{режим}"));
            Ok(ЗначениеV3::Целое(fd))
        });
        ф.регистрация("zip_file_write", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let данные = арг.get(1).map(как_строка).unwrap_or_default();
            std::fs::write(&путь, данные.as_bytes()).map_err(|e| e.to_string())?;
            Ok(ЗначениеV3::Целое(данные.len() as i64))
        });
        ф.регистрация("zip_file_append", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            let данные = арг.get(1).map(как_строка).unwrap_or_default();
            use std::fs::OpenOptions;
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&путь)
                .map_err(|e| e.to_string())?;
            f.write_all(данные.as_bytes()).map_err(|e| e.to_string())?;
            Ok(ЗначениеV3::Целое(данные.len() as i64))
        });
        ф.регистрация("zip_file_read", |арг| {
            let путь = арг.first().map(как_строка).unwrap_or_default();
            match std::fs::read_to_string(&путь) {
                Ok(s) => Ok(ЗначениеV3::Строка(s)),
                Err(_) => Ok(ЗначениеV3::Целое(0)),
            }
        });

        ф
    }

    fn регистрация(&mut self, имя: &str, функция: ВстроеннаяФункция) {
        self.функции.insert(имя.to_string(), функция);
    }

    pub fn вызвать(&self, имя: &str, аргументы: &[ЗначениеV3]) -> Result<ЗначениеV3, String> {
        let функция = self
            .функции
            .get(имя)
            .ok_or_else(|| format!("Встроенная функция '{имя}' не найдена"))?;
        функция(аргументы)
    }

    pub fn существует(&self, имя: &str) -> bool {
        self.функции.contains_key(имя)
    }
}
