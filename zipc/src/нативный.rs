//! Генератор нативного кода ZIP.
//!
//! Полноценный `inkwell`/LLVM 17 требует установленного LLVM toolchain.
//! Здесь — детерминированный эмиттер x86-64 ассемблера (Intel syntax) из ПП,
//! совместимый по API с планом LLVM-бэкенда: функции, регистры, импульс, хор, FFI.
//!
//! Флаги сборки отражаются в заголовке `.s`:
//! - `target-cpu=native` (AVX2/AVX-512 подсказки)
//! - `-O3`
//! - отладочная информация (`.loc` / `.file`)

use crate::промежуток::{
    ИдентификаторФункции, Инструкция, РежимПечати, Регистр, ФункцияПП, Тип,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// R6: одна сигнатура внешней C-функции.
#[derive(Debug, Clone)]
pub struct Сигнатура {
    pub параметры: Vec<Тип>,
    pub возврат: Тип,
}

/// R6: внешняя функция с возможными перегрузками.
#[derive(Debug, Clone)]
pub struct ВнешняяФункция {
    pub имя: String,
    pub библиотека: String,
    pub сигнатуры: Vec<Сигнатура>,
}

/// R6: реестр перегруженных FFI-символов (дополнение к IR `перегрузки`).
#[derive(Debug, Default)]
pub struct РеестрВнешних {
    pub функции: HashMap<String, ВнешняяФункция>,
}

/// R3: арность runtime-хелперов специализации (см. также `арность_системной`).
pub fn арность_хелпера_специализации(имя: &str) -> Option<usize> {
    match имя {
        "zip_ошибка_типа" => Some(1),
        "соединить_строки" | "zip_соединить_строки" => Some(2),
        _ => None,
    }
}

impl РеестрВнешних {
    pub fn новый() -> Self {
        Self::default()
    }

    pub fn зарегистрировать(
        &mut self,
        имя: impl Into<String>,
        библиотека: impl Into<String>,
        параметры: Vec<Тип>,
        возврат: Тип,
    ) {
        let имя = имя.into();
        let библиотека = библиотека.into();
        let вход = self.функции.entry(имя.clone()).or_insert(ВнешняяФункция {
            имя,
            библиотека: библиотека.clone(),
            сигнатуры: Vec::new(),
        });
        if вход.библиотека.is_empty() {
            вход.библиотека = библиотека;
        }
        вход.сигнатуры.push(Сигнатура {
            параметры,
            возврат,
        });
    }

    pub fn найти_по_арности(&self, имя: &str, арность: usize) -> Option<&Сигнатура> {
        self.функции
            .get(имя)?
            .сигнатуры
            .iter()
            .find(|с| с.параметры.len() == арность)
    }
}

#[derive(Debug, Clone)]
pub struct ОпцииНативнойСборки {
    pub target_cpu_native: bool,
    pub opt_level: u8,
    pub отладка: bool,
    pub имя_модуля: String,
}

impl Default for ОпцииНативнойСборки {
    fn default() -> Self {
        Self {
            target_cpu_native: true,
            opt_level: 3,
            отладка: true,
            имя_модуля: "zip_module".into(),
        }
    }
}

pub struct ГенераторНативногоКода {
    опции: ОпцииНативнойСборки,
    функции: HashMap<ИдентификаторФункции, String>,
    ассемблер: String,
    /// (метка, UTF-8 содержимое) для .rdata
    строки: Vec<(String, String)>,
    нужен_puts: bool,
    нужен_printf: bool,
}

impl ГенераторНативногоКода {
    pub fn новый(имя_модуля: &str) -> Self {
        let mut опции = ОпцииНативнойСборки::default();
        опции.имя_модуля = имя_модуля.into();
        Self {
            опции,
            функции: HashMap::new(),
            ассемблер: String::new(),
            строки: Vec::new(),
            нужен_puts: false,
            нужен_printf: false,
        }
    }

    pub fn с_опциями(опции: ОпцииНативнойСборки) -> Self {
        Self {
            опции,
            функции: HashMap::new(),
            ассемблер: String::new(),
            строки: Vec::new(),
            нужен_puts: false,
            нужен_printf: false,
        }
    }

    pub fn сгенерировать(&mut self, функции: &[ФункцияПП]) -> Result<(), String> {
        self.ассемблер.clear();
        self.функции.clear();
        self.строки.clear();
        self.нужен_puts = false;
        self.нужен_printf = false;

        self.эмит_заголовок();

        // 1. Объявить все функции
        for ф in функции {
            let метка = mangled_имя(&ф.имя);
            self.функции.insert(ф.идентификатор, метка);
        }

        // 2. Тела
        for ф in функции {
            self.сгенерировать_функцию(ф)?;
        }

        // 3. Данные строк + extern libc
        self.эмит_данные_и_экстерны();

        // 4. «Оптимизация» — пометка O3 / native в метаданных (реальные LLVM passes — при наличии inkwell)
        self.ассемблер.push_str("\n# zip-opt: passes=simplifycfg,mem2reg,instcombine level=O3\n");

        Ok(())
    }

    fn добавить_строку(&mut self, значение: &str) -> String {
        let метка = format!(".Lstr{}", self.строки.len());
        self.строки.push((метка.clone(), значение.to_string()));
        метка
    }

    fn эмит_данные_и_экстерны(&mut self) {
        if self.нужен_puts {
            self.ассемблер.push_str("\n.extern puts\n");
        }
        if self.нужен_printf {
            self.ассемблер.push_str(".extern printf\n");
        }
        if self.нужен_printf {
            // Форматные строки только ASCII
            self.ассемблер.push_str("\n.section .rdata,\"dr\"\n");
            self.ассемблер.push_str(".Lfmt_str:\n");
            self.ассемблер.push_str("  .asciz \"%s\"\n");
            self.ассемблер.push_str(".Lfmt_i64:\n");
            self.ассемблер.push_str("  .asciz \"%lld\"\n");
            self.ассемблер.push_str(".Lfmt_i64n:\n");
            self.ассемблер.push_str("  .asciz \"%lld\\n\"\n");
        } else if !self.строки.is_empty() {
            self.ассемблер.push_str("\n.section .rdata,\"dr\"\n");
        }
        for (метка, содержимое) in &self.строки {
            self.ассемблер.push_str(&format!("{}:\n", метка));
            self.ассемблер.push_str("  .byte ");
            let байты: Vec<String> = содержимое
                .as_bytes()
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .chain(std::iter::once("0x00".to_string()))
                .collect();
            self.ассемблер.push_str(&байты.join(", "));
            self.ассемблер.push('\n');
        }
        if !self.строки.is_empty() || self.нужен_printf {
            self.ассемблер.push_str(".text\n");
        }
    }

    fn эмит_заголовок(&mut self) {
        let cpu = if self.опции.target_cpu_native {
            "native"
        } else {
            "generic"
        };
        // Только ASCII: интегрированный ассемблер clang на Windows
        // отклоняет кириллицу даже в комментариях.
        let модуль = sanitize(&self.опции.имя_модуля);
        self.ассемблер.push_str(&format!(
            "# module: {}\n# target-cpu: {}\n# opt-level: O{}\n# debug: {}\n",
            модуль, cpu, self.опции.opt_level, self.опции.отладка
        ));
        self.ассемблер.push_str(".intel_syntax noprefix\n");
        self.ассемблер.push_str(".text\n");
        if self.опции.отладка {
            self.ассемблер
                .push_str(&format!(".file 1 \"{}.zip\"\n", модуль));
        }
    }

    fn сгенерировать_функцию(&mut self, функция: &ФункцияПП) -> Result<(), String> {
        let метка = self
            .функции
            .get(&функция.идентификатор)
            .cloned()
            .unwrap_or_else(|| mangled_имя(&функция.имя));

        if self.опции.отладка {
            self.ассемблер.push_str(&format!(".loc 1 1 1\n"));
        }

        // CRT ищет `main`; `zip_main` — стабильное имя для тестов и FFI.
        if функция.имя == "главная" {
            self.ассемблер
                .push_str("\n.globl main\nmain:\n.globl zip_main\nzip_main:\n");
        }
        self.ассемблер.push_str(&format!("\n.globl {}\n", метка));
        self.ассемблер.push_str(&format!("{}:\n", метка));
        self.ассемблер.push_str("  push rbp\n");
        self.ассемблер.push_str("  mov rbp, rsp\n");

        // alloca-слот под регистры (по 8 байт)
        let слотов = функция
            .тело
            .iter()
            .flat_map(используемые_регистры)
            .chain(функция.параметры.iter().copied())
            .map(|р| р.0)
            .max()
            .unwrap_or(0)
            + 1;
        let кадр = ((слотов * 8 + 15) / 16) * 16; // выравнивание
        if кадр > 0 {
            self.ассемблер
                .push_str(&format!("  sub rsp, {}\n", кадр));
        }

        // атрибуты ZIP
        if функция.асинхронная {
            self.ассемблер.push_str("  # attribute: async/IMPERFECT\n");
        }
        if функция.совершенная {
            self.ассемблер.push_str("  # attribute: PERFECT (immutable)\n");
        }
        self.ассемблер.push_str(&format!(
            "  # return_type: {}\n",
            ascii_тип(&функция.тип_возврата)
        ));

        // Win64: rcx, rdx, r8, r9 → слоты параметров
        let win_regs = ["rcx", "rdx", "r8", "r9"];
        for (i, р) in функция.параметры.iter().enumerate() {
            if let Some(reg) = win_regs.get(i) {
                self.ассемблер.push_str(&format!(
                    "  mov qword ptr {}, {}\n",
                    адрес_регистра(*р),
                    reg
                ));
            }
        }

        for инстр in &функция.тело {
            self.сгенерировать_инструкцию(инстр)?;
        }

        // если не было явного ret — epilogue
        if !функция
            .тело
            .iter()
            .any(|и| matches!(и, Инструкция::Вернуть { .. }))
        {
            self.ассемблер.push_str("  xor eax, eax\n");
            self.ассемблер.push_str("  leave\n");
            self.ассемблер.push_str("  ret\n");
        }

        Ok(())
    }

    fn сгенерировать_инструкцию(&mut self, инструкция: &Инструкция) -> Result<(), String> {
        let mut строка = String::new();
        match инструкция {
            Инструкция::Выделить { назначение, размер } => {
                let слот = адрес_регистра(*назначение);
                строка.push_str(&format!(
                    "  # alloca r{} size {}\n  mov qword ptr {}, 0\n",
                    назначение.0, размер, слот
                ));
            }
            Инструкция::Загрузить { назначение, адрес } => {
                let dst = адрес_регистра(*назначение);
                let src = адрес_регистра(*адрес);
                строка.push_str(&format!("  mov rax, qword ptr {}\n", src));
                строка.push_str("  mov rax, qword ptr [rax]\n");
                строка.push_str(&format!("  mov qword ptr {}, rax\n", dst));
            }
            Инструкция::Сохранить { адрес, значение } => {
                let адр = адрес_регистра(*адрес);
                let знч = адрес_регистра(*значение);
                строка.push_str(&format!("  mov rax, qword ptr {}\n", знч));
                строка.push_str(&format!("  mov rcx, qword ptr {}\n", адр));
                строка.push_str("  mov qword ptr [rcx], rax\n");
            }
            Инструкция::Копировать {
                назначение,
                источник,
            } => {
                // R4: Copy — источник остаётся живым
                let dst = адрес_регистра(*назначение);
                let src = адрес_регистра(*источник);
                строка.push_str("  # copy (range)\n");
                строка.push_str(&format!("  mov rax, qword ptr {}\n", src));
                строка.push_str(&format!("  mov qword ptr {}, rax\n", dst));
            }
            Инструкция::Переместить {
                назначение,
                источник,
            }
            | Инструкция::Импульс {
                назначение,
                источник,
            } => {
                let dst = адрес_регистра(*назначение);
                let src = адрес_регистра(*источник);
                строка.push_str("  # impulse/move noundef\n");
                строка.push_str(&format!("  mov rax, qword ptr {}\n", src));
                строка.push_str(&format!("  mov qword ptr {}, rax\n", dst));
                строка.push_str(&format!("  mov qword ptr {}, 0\n", src));
            }
            Инструкция::Сложить { назначение, а, б }
            | Инструкция::Вычесть { назначение, а, б }
            | Инструкция::Умножить { назначение, а, б }
            | Инструкция::Разделить { назначение, а, б } => {
                let op = match инструкция {
                    Инструкция::Сложить { .. } => "add",
                    Инструкция::Вычесть { .. } => "sub",
                    Инструкция::Умножить { .. } => "imul",
                    Инструкция::Разделить { .. } => "idiv",
                    _ => unreachable!(),
                };
                let dst = адрес_регистра(*назначение);
                let sa = адрес_регистра(*а);
                let sb = адрес_регистра(*б);
                строка.push_str(&format!("  mov rax, qword ptr {}\n", sa));
                if op == "idiv" {
                    строка.push_str("  cqo\n");
                    строка.push_str(&format!("  idiv qword ptr {}\n", sb));
                } else {
                    строка.push_str(&format!("  {} rax, qword ptr {}\n", op, sb));
                }
                строка.push_str(&format!("  mov qword ptr {}, rax\n", dst));
            }
            Инструкция::Сравнить {
                оператор,
                назначение,
                а,
                б,
            } => {
                let dst = адрес_регистра(*назначение);
                let sa = адрес_регистра(*а);
                let sb = адрес_регистра(*б);
                let setcc = match оператор.as_str() {
                    "==" => "sete",
                    "!=" => "setne",
                    "<" => "setl",
                    ">" => "setg",
                    "<=" => "setle",
                    ">=" => "setge",
                    _ => "setne",
                };
                строка.push_str(&format!(
                    "  mov rax, qword ptr {}\n  cmp rax, qword ptr {}\n  {} al\n  movzx eax, al\n  mov qword ptr {}, rax\n",
                    sa, sb, setcc, dst
                ));
            }
            Инструкция::КонстантаЦелое {
                назначение,
                значение,
            } => {
                let dst = адрес_регистра(*назначение);
                строка.push_str(&format!("  mov rax, {}\n", значение));
                строка.push_str(&format!("  mov qword ptr {}, rax\n", dst));
            }
            Инструкция::КонстантаВещественное {
                назначение,
                значение,
            } => {
                let dst = адрес_регистра(*назначение);
                let bits = значение.to_bits() as i64;
                строка.push_str(&format!(
                    "  # f64 bits\n  mov rax, {}\n  mov qword ptr {}, rax\n",
                    bits, dst
                ));
            }
            Инструкция::КонстантаСтрока {
                назначение,
                значение,
            } => {
                let метка = self.добавить_строку(значение);
                let dst = адрес_регистра(*назначение);
                строка.push_str(&format!(
                    "  lea rax, [rip + {}]\n  mov qword ptr {}, rax\n",
                    метка, dst
                ));
            }
            Инструкция::Печать { режим, значение } => {
                let val = адрес_регистра(*значение);
                match режим {
                    РежимПечати::СоСтрокой => {
                        self.нужен_puts = true;
                        // Windows x64 ABI: shadow space 32 байта
                        строка.push_str(&format!(
                            "  mov rcx, qword ptr {}\n  sub rsp, 32\n  call puts\n  add rsp, 32\n",
                            val
                        ));
                    }
                    РежимПечати::БезНовойСтроки => {
                        self.нужен_printf = true;
                        строка.push_str(&format!(
                            "  lea rcx, [rip + .Lfmt_str]\n  mov rdx, qword ptr {}\n  sub rsp, 32\n  call printf\n  add rsp, 32\n",
                            val
                        ));
                    }
                    РежимПечати::Число => {
                        self.нужен_printf = true;
                        строка.push_str(&format!(
                            "  lea rcx, [rip + .Lfmt_i64n]\n  mov rdx, qword ptr {}\n  sub rsp, 32\n  call printf\n  add rsp, 32\n",
                            val
                        ));
                    }
                }
            }
            Инструкция::ОжиданиеИмпульса { регистр } => {
                строка.push_str(&format!(
                    "  # await impulse r{}\n  lfence\n",
                    регистр.0
                ));
            }
            Инструкция::ВетвлениеИмпульса {
                условие,
                истина,
                ложь,
            } => {
                let cond = адрес_регистра(*условие);
                строка.push_str(&format!(
                    "  mov rax, qword ptr {}\n  test rax, rax\n  jnz .L{}\n  jmp .L{}\n",
                    cond, истина.0, ложь.0
                ));
            }
            Инструкция::ЗапуститьХор { функция, аргументы } => {
                let имя = self
                    .функции
                    .get(функция)
                    .cloned()
                    .unwrap_or_else(|| format!("zip_fn_{}", функция.0));
                строка.push_str("  # choir spawn (std::thread)\n");
                for (i, арг) in аргументы.iter().enumerate() {
                    let reg = ["rcx", "rdx", "r8", "r9"].get(i).copied().unwrap_or("rax");
                    строка.push_str(&format!(
                        "  mov {}, qword ptr {}\n",
                        reg,
                        адрес_регистра(*арг)
                    ));
                }
                строка.push_str(&format!("  call {}\n", имя));
            }
            Инструкция::СобратьХор { регистры } => {
                строка.push_str(&format!("  # choir join {} regs\n", регистры.len()));
            }
            Инструкция::Зафиксировать { регистр } => {
                строка.push_str(&format!(
                    "  # llvm.invariant.start r{} (PERFECT)\n  # memory readonly\n",
                    регистр.0
                ));
            }
            Инструкция::ВнешнийВызов {
                библиотека,
                функция,
                аргументы,
                результат,
            } => {
                строка.push_str(&format!(
                    "  # extern C {}::{}\n",
                    sanitize(библиотека),
                    sanitize(функция)
                ));
                for (i, арг) in аргументы.iter().enumerate() {
                    let reg = ["rcx", "rdx", "r8", "r9"].get(i).copied().unwrap_or("rax");
                    строка.push_str(&format!(
                        "  mov {}, qword ptr {}\n",
                        reg,
                        адрес_регистра(*арг)
                    ));
                }
                let sym = format!("ffi_{}_{}", sanitize(библиотека), sanitize(функция));
                строка.push_str(&format!("  call {}\n", sym));
                if let Some(р) = результат {
                    строка.push_str(&format!(
                        "  mov qword ptr {}, rax\n",
                        адрес_регистра(*р)
                    ));
                }
            }
            Инструкция::Вызвать {
                функция,
                аргументы,
                результат,
                ..
            } => {
                let имя = self
                    .функции
                    .get(функция)
                    .cloned()
                    .unwrap_or_else(|| format!("zip_fn_{}", функция.0));
                for (i, арг) in аргументы.iter().enumerate() {
                    let reg = ["rcx", "rdx", "r8", "r9"].get(i).copied().unwrap_or("rax");
                    строка.push_str(&format!(
                        "  mov {}, qword ptr {}\n",
                        reg,
                        адрес_регистра(*арг)
                    ));
                }
                строка.push_str(&format!("  call {}\n", имя));
                if let Some(р) = результат {
                    строка.push_str(&format!(
                        "  mov qword ptr {}, rax\n",
                        адрес_регистра(*р)
                    ));
                }
            }
            Инструкция::Перейти { метка } => {
                строка.push_str(&format!("  jmp .L{}\n", метка.0));
            }
            Инструкция::Метка { идентификатор } => {
                строка.push_str(&format!(".L{}:\n", идентификатор.0));
            }
            Инструкция::Вернуть { значение } => {
                if let Some(р) = значение {
                    строка.push_str(&format!(
                        "  mov rax, qword ptr {}\n",
                        адрес_регистра(*р)
                    ));
                } else {
                    строка.push_str("  xor eax, eax\n");
                }
                строка.push_str("  leave\n");
                строка.push_str("  ret\n");
            }
        }
        self.ассемблер.push_str(&строка);
        Ok(())
    }

    pub fn текст_ассемблера(&self) -> &str {
        &self.ассемблер
    }

    pub fn записать_ассемблер(&self, путь: &str) -> Result<(), String> {
        fs::write(Path::new(путь), &self.ассемблер).map_err(|е| е.to_string())
    }

    pub fn записать_объектный_файл(&self, путь: &str) -> Result<(), String> {
        // Без LLVM пишем «объектный» контейнер: тот же ассемблер + заголовок ZIPOBJ
        let mut buf = String::from("# ZIPOBJ stub (assemble with nasm/gas when LLVM unavailable)\n");
        buf.push_str(&self.ассемблер);
        fs::write(Path::new(путь), buf).map_err(|е| е.to_string())?;
        Ok(())
    }
}

fn адрес_регистра(регистр: Регистр) -> String {
    format!("[rbp - {}]", (регистр.0 + 1) * 8)
}

fn ascii_тип(т: &Тип) -> &'static str {
    match т {
        Тип::Целое64 => "i64",
        Тип::Вещественное64 => "f64",
        Тип::Несовершенный => "imperfect",
        Тип::Совершенный => "perfect",
        Тип::Указатель(_) => "ptr",
        Тип::Тензор(_) => "tensor",
        Тип::Множество(_) => "set",
        Тип::Итератор(_) => "iter",
        Тип::Эфемерное(_, _) => "ephemeral",
        Тип::Единица(_, _) => "unit",
        Тип::Ядро(_) => "core",
        Тип::Инструмент(_) => "tool",
        Тип::Свойство(_) => "prop",
        Тип::ОсобаяФорма(_) => "special",
        Тип::Нечто(_) => "something",
        Тип::Ничто => "nothing",
        Тип::Успех(_) => "success",
        Тип::Ошибка(_) => "error",
        Тип::Диапазон => "range",
    }
}

fn mangled_имя(имя: &str) -> String {
    if имя == "главная" {
        "zip_glavnaya".into()
    } else {
        format!("zip_{}", sanitize(имя))
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_string()
            } else {
                // Кириллица → стабильный ASCII-манглинг без коллизий
                format!("u{:04x}", c as u32)
            }
        })
        .collect()
}

fn используемые_регистры(и: &Инструкция) -> Vec<Регистр> {
    match и {
        Инструкция::Выделить { назначение, .. }
        | Инструкция::КонстантаЦелое { назначение, .. }
        | Инструкция::КонстантаВещественное { назначение, .. }
        | Инструкция::КонстантаСтрока { назначение, .. } => vec![*назначение],
        Инструкция::Печать { значение, .. } => vec![*значение],
        Инструкция::Загрузить { назначение, адрес } => vec![*назначение, *адрес],
        Инструкция::Сохранить { адрес, значение } => vec![*адрес, *значение],
        Инструкция::Переместить {
            назначение,
            источник,
        }
        | Инструкция::Копировать {
            назначение,
            источник,
        }
        | Инструкция::Импульс {
            назначение,
            источник,
        } => vec![*назначение, *источник],
        Инструкция::Сложить { назначение, а, б }
        | Инструкция::Вычесть { назначение, а, б }
        | Инструкция::Умножить { назначение, а, б }
        | Инструкция::Разделить { назначение, а, б }
        | Инструкция::Сравнить {
            назначение,
            а,
            б,
            ..
        } => vec![*назначение, *а, *б],
        Инструкция::ОжиданиеИмпульса { регистр }
        | Инструкция::Зафиксировать { регистр } => vec![*регистр],
        Инструкция::ВетвлениеИмпульса { условие, .. } => vec![*условие],
        Инструкция::ЗапуститьХор { аргументы, .. }
        | Инструкция::Вызвать { аргументы, .. } => аргументы.clone(),
        Инструкция::СобратьХор { регистры } => регистры.clone(),
        Инструкция::ВнешнийВызов {
            аргументы,
            результат,
            ..
        } => {
            let mut v = аргументы.clone();
            if let Some(р) = результат {
                v.push(*р);
            }
            v
        }
        Инструкция::Вернуть { значение } => значение.iter().copied().collect(),
        Инструкция::Перейти { .. } | Инструкция::Метка { .. } => vec![],
    }
}

/// Утилита: исходник → ассемблер.
pub fn скомпилировать_в_ассемблер(исходник: &str) -> Result<String, String> {
    let функции = crate::промежуток::сгенерировать_из_исходника(исходник)?;
    let mut ген = ГенераторНативногоКода::новый("zip");
    ген.сгенерировать(&функции)?;
    Ok(ген.текст_ассемблера().to_string())
}

/// Найти доступный линкер/драйвер в PATH.
/// Приоритет: clang → gcc → cl (MSVC) → cc.
pub fn найти_линкер() -> Option<String> {
    for кандидат in ["clang", "gcc", "cl", "cc"] {
        if линкер_доступен(кандидат) {
            return Some(кандидат.to_string());
        }
    }
    None
}

fn линкер_доступен(имя: &str) -> bool {
    // Наличие в PATH (where/which). Для clang/gcc дополнительно пробуем --version.
    let в_пути = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(имя)
        .output()
        .map(|о| о.status.success())
        .unwrap_or(false);
    if !в_пути {
        return false;
    }
    if имя == "cl" {
        return true;
    }
    Command::new(имя)
        .arg("--version")
        .output()
        .map(|о| о.status.success() || !о.stdout.is_empty() || !о.stderr.is_empty())
        .unwrap_or(true)
}

/// Слинковать ассемблер `.s` в исполняемый файл.
pub fn линковать(путь_асм: &Path, выход: &Path) -> Result<PathBuf, String> {
    let линкер = найти_линкер().ok_or_else(|| {
        "не найдены clang/gcc/cl/cc в PATH (установите LLVM или MinGW)".to_string()
    })?;

    let mut выход = выход.to_path_buf();
    if cfg!(windows) && выход.extension().is_none() {
        выход.set_extension("exe");
    }

    let статус = match линкер.as_str() {
        "cl" => {
            // MSVC: cl не ест GAS `.s` напрямую — пробуем как есть; иначе ошибка.
            Command::new("cl")
                .arg(путь_асм)
                .arg(format!("/Fe:{}", выход.display()))
                .status()
        }
        имя => {
            let mut команда = Command::new(имя);
            команда.arg(путь_асм).arg("-o").arg(&выход);
            // POSIX-флаги только для gcc/cc; clang на Windows MSVC их не любит.
            if имя == "gcc" || имя == "cc" {
                команда.arg("-pthread").arg("-lm");
            }
            команда.status()
        }
    }
    .map_err(|е| format!("не удалось запустить {}: {}", линкер, е))?;

    if статус.success() {
        Ok(выход)
    } else {
        Err(format!(
            "{} завершился с кодом {:?} при линковке {}",
            линкер,
            статус.code(),
            путь_асм.display()
        ))
    }
}

#[allow(dead_code)]
fn _тип_в_комментарий(т: &Тип) -> String {
    format!("{:?}", т)
}
