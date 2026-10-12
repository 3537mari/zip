//! Полный генератор ZIPB — алгоритм на стороне VM (без C-bootstrap).

use crate::байткод_v3::{сериализовать_v3, ФункцияV3, МодульV3, Опкод};
use crate::встроенные_v3::Встроенные;
use crate::исполнитель_v3::ЗначениеV3;
use crate::zip_ast::{Параметр, Узел};
use crate::zip_parse_полный;
use std::collections::HashMap;
use std::sync::Mutex;

struct Код {
    bytes: Vec<u8>,
    ipc: u32,
}

impl Код {
    fn новый() -> Self {
        Self {
            bytes: Vec::new(),
            ipc: 0,
        }
    }

    fn op0(&mut self, op: Опкод) {
        self.bytes.push(op as u8);
        self.ipc += 1;
    }

    fn op_i64(&mut self, op: Опкод, v: i64) {
        self.bytes.push(op as u8);
        self.bytes.extend_from_slice(&v.to_le_bytes());
        self.ipc += 1;
    }

    fn op_f64(&mut self, op: Опкод, v: f64) {
        self.bytes.push(op as u8);
        self.bytes.extend_from_slice(&v.to_le_bytes());
        self.ipc += 1;
    }

    fn op_u32(&mut self, op: Опкод, v: u32) {
        self.bytes.push(op as u8);
        self.bytes.extend_from_slice(&v.to_le_bytes());
        self.ipc += 1;
    }

    fn op_u16(&mut self, op: Опкод, v: u16) {
        self.bytes.push(op as u8);
        self.bytes.extend_from_slice(&v.to_le_bytes());
        self.ipc += 1;
    }

    fn op_u16x2(&mut self, op: Опкод, a: u16, b: u16) {
        self.bytes.push(op as u8);
        self.bytes.extend_from_slice(&a.to_le_bytes());
        self.bytes.extend_from_slice(&b.to_le_bytes());
        self.ipc += 1;
    }

    /// JUMP* + placeholder; возвращает байтовое смещение u32 для патча.
    fn emit_jump(&mut self, op: Опкод) -> usize {
        self.bytes.push(op as u8);
        let at = self.bytes.len();
        self.bytes.extend_from_slice(&0u32.to_le_bytes());
        self.ipc += 1;
        at
    }

    fn patch_u32(&mut self, at: usize, v: u32) {
        self.bytes[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
}

pub struct Генератор {
    функции_имена: Vec<String>,
    функции_арность: Vec<u16>,
    функции_код: Vec<Vec<u8>>,
    функции_локалы: Vec<u16>,
    строки: Vec<String>,
    индекс_строк: HashMap<String, u32>,
}

impl Генератор {
    pub fn новый() -> Self {
        Self {
            функции_имена: Vec::new(),
            функции_арность: Vec::new(),
            функции_код: Vec::new(),
            функции_локалы: Vec::new(),
            строки: Vec::new(),
            индекс_строк: HashMap::new(),
        }
    }

    fn добавить_строку(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.индекс_строк.get(s) {
            return i;
        }
        let i = self.строки.len() as u32;
        self.строки.push(s.to_string());
        self.индекс_строк.insert(s.to_string(), i);
        i
    }

    fn найти_функцию(&self, имя: &str) -> Option<u32> {
        self.функции_имена
            .iter()
            .position(|n| n == имя)
            .map(|i| i as u32)
    }

    pub fn сгенерировать_программу(&mut self, узел: &Узел) -> Result<Vec<u8>, String> {
        let объявления = match узел {
            Узел::Программа { объявления } => объявления,
            _ => return Err("Ожидалась программа".into()),
        };

        // Pass 1: таблица функций (+ имена в строках)
        for об in объявления {
            if let Узел::Функция {
                имя, параметры, ..
            } = об
            {
                self.добавить_строку(имя);
                self.функции_имена.push(имя.clone());
                self.функции_арность.push(параметры.len() as u16);
                self.функции_код.push(Vec::new());
                self.функции_локалы.push(16);
            }
        }

        // Pass 1b: внешние вызовы как stub-функции (как C collect_externals)
        self.собрать_внешние(узел);

        if self.функции_имена.is_empty() {
            self.добавить_строку("главная");
            self.функции_имена.push("главная".into());
            self.функции_арность.push(0);
            let mut k = Код::новый();
            k.op_i64(Опкод::PushI64, 0);
            k.op0(Опкод::Ret);
            self.функции_код.push(k.bytes);
            self.функции_локалы.push(16);
        } else {
            // Pass 2: тела
            for об in объявления {
                if let Узел::Функция {
                    имя,
                    параметры,
                    тело,
                    ..
                } = об
                {
                    let idx = self
                        .найти_функцию(имя)
                        .ok_or_else(|| format!("нет функции {имя}"))?
                        as usize;
                    let (код, локалы) = self.эмит_функции(параметры, тело)?;
                    self.функции_код[idx] = код;
                    self.функции_локалы[idx] = локалы;
                }
            }
            // Stub для внешних: PUSH 0; RET
            for i in 0..self.функции_имена.len() {
                if self.функции_код[i].is_empty() {
                    let mut k = Код::новый();
                    k.op_i64(Опкод::PushI64, 0);
                    k.op0(Опкод::Ret);
                    self.функции_код[i] = k.bytes;
                    if self.функции_локалы[i] < 16 {
                        self.функции_локалы[i] = 16;
                    }
                }
            }
        }

        let функции = self
            .функции_имена
            .iter()
            .enumerate()
            .map(|(i, имя)| ФункцияV3 {
                имя_индекс: *self.индекс_строк.get(имя).unwrap_or(&0),
                арность: self.функции_арность[i],
                локалы: self.функции_локалы[i],
                флаги: 0,
                код: self.функции_код[i].clone(),
            })
            .collect();

        Ok(сериализовать_v3(&МодульV3 {
            флаги: 0,
            строки: self.строки.clone(),
            функции,
        }))
    }

    fn собрать_внешние(&mut self, узел: &Узел) {
        match узел {
            Узел::Вызов { имя, аргументы } => {
                if self.найти_функцию(имя).is_none() {
                    self.добавить_строку(имя);
                    self.функции_имена.push(имя.clone());
                    self.функции_арность.push(аргументы.len() as u16);
                    self.функции_код.push(Vec::new());
                    self.функции_локалы.push(16);
                }
                for a in аргументы {
                    self.собрать_внешние(a);
                }
            }
            Узел::Программа { объявления }
            | Узел::Блок {
                инструкции: объявления,
            } => {
                for u in объявления {
                    self.собрать_внешние(u);
                }
            }
            Узел::Функция { тело, .. } => self.собрать_внешние(тело),
            Узел::БинарнаяОперация { левый, правый, .. }
            | Узел::ПотокДанных { левый, правый } => {
                self.собрать_внешние(левый);
                self.собрать_внешние(правый);
            }
            Узел::УнарнаяОперация { операнд, .. }
            | Узел::Фиксация {
                выражение: операнд,
            }
            | Узел::Асинхронный {
                выражение: операнд,
            }
            | Узел::Присваивание {
                значение: операнд, ..
            } => self.собрать_внешние(операнд),
            Узел::Если {
                условие,
                тогда,
                иначе,
            } => {
                self.собрать_внешние(условие);
                self.собрать_внешние(тогда);
                if let Some(e) = иначе {
                    self.собрать_внешние(e);
                }
            }
            Узел::Пока { условие, тело } => {
                self.собрать_внешние(условие);
                self.собрать_внешние(тело);
            }
            Узел::Для {
                начало,
                конец,
                тело,
                ..
            } => {
                self.собрать_внешние(начало);
                self.собрать_внешние(конец);
                self.собрать_внешние(тело);
            }
            Узел::Возврат { значение: Some(v) } => self.собрать_внешние(v),
            Узел::Диапазон { начало, конец } => {
                self.собрать_внешние(начало);
                self.собрать_внешние(конец);
            }
            _ => {}
        }
    }

    fn эмит_функции(
        &mut self,
        параметры: &[Параметр],
        тело: &Узел,
    ) -> Result<(Vec<u8>, u16), String> {
        let mut локали: Vec<String> = параметры.iter().map(|p| p.имя.clone()).collect();
        let mut код = Код::новый();
        self.эмит_stmt(тело, &mut код, &mut локали)?;
        if код.bytes.last().copied() != Some(Опкод::Ret as u8) {
            код.op_i64(Опкод::PushI64, 0);
            код.op0(Опкод::Ret);
        }
        let locals = (локали.len().max(16)) as u16;
        Ok((код.bytes, locals))
    }

    fn local_get(локали: &mut Vec<String>, имя: &str) -> u32 {
        if let Some(i) = локали.iter().position(|n| n == имя) {
            return i as u32;
        }
        let i = локали.len() as u32;
        локали.push(имя.to_string());
        i
    }

    fn эмит_stmt(
        &mut self,
        узел: &Узел,
        код: &mut Код,
        локали: &mut Vec<String>,
    ) -> Result<(), String> {
        match узел {
            Узел::Блок { инструкции } => {
                for i in инструкции {
                    self.эмит_stmt(i, код, локали)?;
                }
            }
            Узел::Возврат { значение } => {
                if let Some(v) = значение {
                    self.эмит_expr(v, код, локали)?;
                } else {
                    код.op_i64(Опкод::PushI64, 0);
                }
                код.op0(Опкод::Ret);
            }
            Узел::Присваивание { имя, значение } => {
                self.эмит_expr(значение, код, локали)?;
                let slot = Self::local_get(локали, имя);
                код.op_u32(Опкод::Store, slot);
            }
            Узел::Если {
                условие,
                тогда,
                иначе,
            } => {
                self.эмит_expr(условие, код, локали)?;
                let jelse = код.emit_jump(Опкод::JumpIfNot);
                self.эмит_stmt(тогда, код, локали)?;
                let jend = код.emit_jump(Опкод::Jump);
                код.patch_u32(jelse, код.ipc);
                if let Some(e) = иначе {
                    self.эмит_stmt(e, код, локали)?;
                }
                код.patch_u32(jend, код.ipc);
            }
            Узел::Пока { условие, тело } => {
                let loop_ipc = код.ipc;
                self.эмит_expr(условие, код, локали)?;
                let jend = код.emit_jump(Опкод::JumpIfNot);
                self.эмит_stmt(тело, код, локали)?;
                let jback = код.emit_jump(Опкод::Jump);
                код.patch_u32(jback, loop_ipc);
                код.patch_u32(jend, код.ipc);
            }
            Узел::Для {
                переменная,
                начало,
                конец,
                тело,
            } => {
                let slot = Self::local_get(локали, переменная);
                self.эмит_expr(начало, код, локали)?;
                код.op_u32(Опкод::Store, slot);
                let loop_ipc = код.ipc;
                код.op_u32(Опкод::Load, slot);
                self.эмит_expr(конец, код, локали)?;
                код.op0(Опкод::Lt);
                let jend = код.emit_jump(Опкод::JumpIfNot);
                self.эмит_stmt(тело, код, локали)?;
                код.op_u32(Опкод::Load, slot);
                код.op_i64(Опкод::PushI64, 1);
                код.op0(Опкод::Add);
                код.op_u32(Опкод::Store, slot);
                let jback = код.emit_jump(Опкод::Jump);
                код.patch_u32(jback, loop_ipc);
                код.patch_u32(jend, код.ipc);
            }
            Узел::Прервать | Узел::Продолжить => {}
            Узел::Директива { .. } => {}
            other => {
                self.эмит_expr(other, код, локали)?;
            }
        }
        Ok(())
    }

    fn эмит_expr(
        &mut self,
        узел: &Узел,
        код: &mut Код,
        локали: &mut Vec<String>,
    ) -> Result<(), String> {
        match узел {
            Узел::Константа {
                значение_ц,
                значение_в,
                значение_с,
            } => {
                if let Some(s) = значение_с {
                    if s == "истина" {
                        код.op0(Опкод::PushTrue);
                    } else if s == "ложь" {
                        код.op0(Опкод::PushFalse);
                    } else if значение_ц.is_none() && значение_в.is_none() {
                        let i = self.добавить_строку(s);
                        код.op_u32(Опкод::PushStr, i);
                    } else if let Some(c) = значение_ц {
                        код.op_i64(Опкод::PushI64, *c);
                    } else if let Some(f) = значение_в {
                        код.op_f64(Опкод::PushF64, *f);
                    } else {
                        let i = self.добавить_строку(s);
                        код.op_u32(Опкод::PushStr, i);
                    }
                } else if let Some(f) = значение_в {
                    код.op_f64(Опкод::PushF64, *f);
                } else if let Some(c) = значение_ц {
                    код.op_i64(Опкод::PushI64, *c);
                } else {
                    код.op0(Опкод::PushNull);
                }
            }
            Узел::Идентификатор { имя } => {
                let slot = Self::local_get(локали, имя);
                код.op_u32(Опкод::Load, slot);
            }
            Узел::БинарнаяОперация {
                оператор,
                левый,
                правый,
            } => {
                self.эмит_expr(левый, код, локали)?;
                self.эмит_expr(правый, код, локали)?;
                let op = match оператор.as_str() {
                    "+" => Опкод::Add,
                    "-" => Опкод::Sub,
                    "*" => Опкод::Mul,
                    "/" => Опкод::Div,
                    "%" => Опкод::Mod,
                    "==" => Опкод::Eq,
                    "!=" => Опкод::Ne,
                    "<" => Опкод::Lt,
                    "<=" => Опкод::Le,
                    ">" => Опкод::Gt,
                    ">=" => Опкод::Ge,
                    "||" => {
                        код.op0(Опкод::ChoirJoin);
                        return Ok(());
                    }
                    "и" | "или" => Опкод::Eq, // упрощение
                    _ => return Err(format!("оператор {оператор}")),
                };
                код.op0(op);
            }
            Узел::УнарнаяОперация { оператор, операнд } => {
                if оператор == "-" {
                    код.op_i64(Опкод::PushI64, 0);
                    self.эмит_expr(операнд, код, локали)?;
                    код.op0(Опкод::Sub);
                } else {
                    self.эмит_expr(операнд, код, локали)?;
                }
            }
            Узел::Вызов { имя, аргументы } => {
                for a in аргументы {
                    self.эмит_expr(a, код, локали)?;
                }
                if let Some(id) = self.найти_функцию(имя) {
                    код.op_u32(Опкод::Call, id);
                } else {
                    let lib = self.добавить_строку("builtin");
                    let fun = self.добавить_строку(имя);
                    код.bytes.push(Опкод::CallNative as u8);
                    код.bytes.extend_from_slice(&lib.to_le_bytes());
                    код.bytes.extend_from_slice(&fun.to_le_bytes());
                    код.ipc += 1;
                }
            }
            Узел::ПотокДанных { левый, правый } => {
                // Как C: два выражения + PULSE r0,r1 (упрощённо)
                self.эмит_expr(левый, код, локали)?;
                код.op_u32(Опкод::Store, 0);
                self.эмит_expr(правый, код, локали)?;
                код.op_u32(Опкод::Store, 1);
                код.op_u16x2(Опкод::Pulse, 0, 1);
            }
            Узел::Фиксация { выражение } => {
                self.эмит_expr(выражение, код, локали)?;
                код.op_u32(Опкод::Store, 0);
                код.op_u16(Опкод::PerfectLock, 0);
            }
            Узел::Асинхронный { выражение } => {
                // ~expr — если вызов, AsyncCall; иначе просто expr
                if let Узел::Вызов { имя, аргументы } = выражение.as_ref() {
                    for a in аргументы {
                        self.эмит_expr(a, код, локали)?;
                    }
                    let id = self.найти_функцию(имя).unwrap_or(0);
                    код.op_u32(Опкод::AsyncCall, id);
                } else {
                    self.эмит_expr(выражение, код, локали)?;
                }
            }
            Узел::Диапазон { начало, конец } => {
                self.эмит_expr(начало, код, локали)?;
                self.эмит_expr(конец, код, локали)?;
            }
            other => {
                // вложенные stmt как expr — игнор / ошибка мягкая
                if matches!(
                    other,
                    Узел::Блок { .. }
                        | Узел::Если { .. }
                        | Узел::Пока { .. }
                        | Узел::Для { .. }
                        | Узел::Возврат { .. }
                        | Узел::Присваивание { .. }
                ) {
                    self.эмит_stmt(other, код, локали)?;
                } else {
                    return Err(format!("неизвестное выражение: {other:?}"));
                }
            }
        }
        Ok(())
    }
}

/// Полная цепочка: лексер → парсер → ZIPB.
pub fn сгенерировать_из_исходника(исходник: &str) -> Result<Vec<u8>, String> {
    let дерево = zip_parse_полный::разобрать_из_исходника(исходник)?;
    let mut г = Генератор::новый();
    г.сгенерировать_программу(&дерево)
}

static LAST_ZIPB: Mutex<Vec<u8>> = Mutex::new(Vec::new());

fn как_строка(з: &ЗначениеV3) -> String {
    match з {
        ЗначениеV3::Строка(s) => s.clone(),
        другой => другой.to_string(),
    }
}

pub fn зарегистрировать(ф: &mut Встроенные) {
    // v6.1: zip_self_gen — только маркер в zip_self_host.rs.
    // Алгоритм генерации для self-host — в ZIP (главное_чистое.zip).
    // zip_gen_полный остаётся для CLI --gen и тестов паритета.
    ф.регистрация_внешняя("zip_gen_полный", |арг| {
        let исходник = арг.first().map(как_строка).unwrap_or_default();
        match сгенерировать_из_исходника(&исходник) {
            Ok(байткод) => {
                let n = байткод.len() as i64;
                *LAST_ZIPB.lock().unwrap() = байткод;
                Ok(ЗначениеV3::Целое(n))
            }
            Err(e) => {
                eprintln!("zip_gen_полный: {e}");
                Ok(ЗначениеV3::Целое(-1))
            }
        }
    });
    ф.регистрация_алиас("генератор_байткод_полный", "zip_gen_полный");
}

#[cfg(test)]
mod тесты {
    use super::*;

    #[test]
    fn магия() {
        let b = сгенерировать_из_исходника("функция главная() { возврат(42) }").unwrap();
        assert_eq!(&b[0..4], b"ZIPB");
        assert_eq!(u16::from_le_bytes([b[4], b[5]]), 3);
    }

    #[test]
    fn арифметика() {
        let b = сгенерировать_из_исходника("функция главная() { возврат(2 + 3) }").unwrap();
        assert!(b.len() > 16);
        assert!(b.contains(&(Опкод::Add as u8)));
    }

    #[test]
    fn если() {
        let b = сгенерировать_из_исходника(
            "функция главная() { если (истина) { возврат(1) } иначе { возврат(0) } }",
        )
        .unwrap();
        assert!(b.contains(&(Опкод::JumpIfNot as u8)));
    }

    #[test]
    fn поток() {
        let b = сгенерировать_из_исходника("функция главная() { САПАР(1) >> ЧАТАВ! }").unwrap();
        assert!(b.contains(&(Опкод::Pulse as u8)));
    }

    #[test]
    fn декодируется() {
        let b = сгенерировать_из_исходника("функция главная() { возврат(42) }").unwrap();
        let м = crate::байткод_v3_декодер::разобрать_v3(&b).unwrap();
        assert!(!м.функции.is_empty());
    }
}
