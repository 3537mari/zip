//! Декодировщик байт-кода ZIP v3.

use crate::байткод::{Инструкция, Модуль, Функция, ВЕРСИЯ as ВЕРСИЯ_VM};
use crate::байткод_v3::{ИнстрV3, МодульV3, ФункцияV3, Опкод, МАГИЯ_V3, ВЕРСИЯ_V3};

#[derive(Debug, Clone)]
pub struct Заголовок {
    pub версия: u16,
    pub флаги: u16,
    pub количество_функций: u32,
    pub смещение_строк: u32,
}

#[derive(Debug)]
pub enum ОшибкаДекодирования {
    НевернаяМагия,
    НевернаяВерсия(u16),
    НеизвестныйОпкод(u8),
    КонецДанных,
    Utf8(String),
}

impl std::fmt::Display for ОшибкаДекодирования {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::НевернаяМагия => write!(f, "неверная магия (ожидалось ZIPB)"),
            Self::НевернаяВерсия(v) => write!(f, "неподдерживаемая версия {}", v),
            Self::НеизвестныйОпкод(b) => write!(f, "неизвестный опкод 0x{:02X}", b),
            Self::КонецДанных => write!(f, "неожиданный конец данных"),
            Self::Utf8(s) => write!(f, "utf-8: {}", s),
        }
    }
}

pub struct Декодировщик<'a> {
    данные: &'a [u8],
    позиция: usize,
}

impl<'a> Декодировщик<'a> {
    pub fn новый(данные: &'a [u8]) -> Self {
        Self {
            данные,
            позиция: 0,
        }
    }

    pub fn прочитать_заголовок(&mut self) -> Result<Заголовок, ОшибкаДекодирования> {
        let магия = self.прочитать_байты(4)?;
        if магия != МАГИЯ_V3 {
            return Err(ОшибкаДекодирования::НевернаяМагия);
        }
        let версия = self.прочитать_u16()?;
        if версия != ВЕРСИЯ_V3 {
            return Err(ОшибкаДекодирования::НевернаяВерсия(версия));
        }
        let флаги = self.прочитать_u16()?;
        let количество_функций = self.прочитать_u32()?;
        let смещение_строк = self.прочитать_u32()?;
        Ok(Заголовок {
            версия,
            флаги,
            количество_функций,
            смещение_строк,
        })
    }

    pub fn прочитать_модуль(&mut self) -> Result<МодульV3, ОшибкаДекодирования> {
        let заг = self.прочитать_заголовок()?;
        self.позиция = заг.смещение_строк as usize;
        let n_str = self.прочитать_u32()? as usize;
        let mut строки = Vec::with_capacity(n_str);
        for _ in 0..n_str {
            let len = self.прочитать_u32()? as usize;
            let b = self.прочитать_байты(len)?;
            let s = String::from_utf8(b.to_vec())
                .map_err(|e| ОшибкаДекодирования::Utf8(e.to_string()))?;
            строки.push(s);
        }
        let mut функции = Vec::with_capacity(заг.количество_функций as usize);
        for _ in 0..заг.количество_функций {
            let имя_индекс = self.прочитать_u32()?;
            let арность = self.прочитать_u16()?;
            let локалы = self.прочитать_u16()?;
            let флаги = self.прочитать_u32()?;
            let code_len = self.прочитать_u32()? as usize;
            let код = self.прочитать_байты(code_len)?.to_vec();
            функции.push(ФункцияV3 {
                имя_индекс,
                арность,
                локалы,
                флаги,
                код,
            });
        }
        Ok(МодульV3 {
            флаги: заг.флаги,
            строки,
            функции,
        })
    }

    pub fn прочитать_опкод(&mut self) -> Result<Опкод, ОшибкаДекодирования> {
        let байт = self.прочитать_u8()?;
        Опкод::из_байта(байт).ok_or(ОшибкаДекодирования::НеизвестныйОпкод(байт))
    }

    pub fn декодировать_инструкции(&mut self) -> Result<Vec<ИнстрV3>, ОшибкаДекодирования> {
        let mut out = Vec::new();
        while self.позиция < self.данные.len() {
            out.push(self.прочитать_инструкцию()?);
        }
        Ok(out)
    }

    fn прочитать_инструкцию(&mut self) -> Result<ИнстрV3, ОшибкаДекодирования> {
        let op = self.прочитать_опкод()?;
        Ok(match op {
            Опкод::Nop => ИнстрV3::Nop,
            Опкод::Pop => ИнстрV3::Pop,
            Опкод::Dup => ИнстрV3::Dup,
            Опкод::Swap => ИнстрV3::Swap,
            Опкод::PushI64 => ИнстрV3::PushI64(self.прочитать_i64()?),
            Опкод::PushF64 => ИнстрV3::PushF64(self.прочитать_f64()?),
            Опкод::PushStr => ИнстрV3::PushStr(self.прочитать_u32()?),
            Опкод::PushNull => ИнстрV3::PushNull,
            Опкод::PushTrue => ИнстрV3::PushTrue,
            Опкод::PushFalse => ИнстрV3::PushFalse,
            Опкод::Add => ИнстрV3::Add,
            Опкод::Sub => ИнстрV3::Sub,
            Опкод::Mul => ИнстрV3::Mul,
            Опкод::Div => ИнстрV3::Div,
            Опкод::Mod => ИнстрV3::Mod,
            Опкод::Eq => ИнстрV3::Eq,
            Опкод::Ne => ИнстрV3::Ne,
            Опкод::Lt => ИнстрV3::Lt,
            Опкод::Le => ИнстрV3::Le,
            Опкод::Gt => ИнстрV3::Gt,
            Опкод::Ge => ИнстрV3::Ge,
            Опкод::Load => ИнстрV3::Load(self.прочитать_u32()?),
            Опкод::Store => ИнстрV3::Store(self.прочитать_u32()?),
            Опкод::Jump => ИнстрV3::Jump(self.прочитать_u32()?),
            Опкод::JumpIf => ИнстрV3::JumpIf(self.прочитать_u32()?),
            Опкод::JumpIfNot => ИнстрV3::JumpIfNot(self.прочитать_u32()?),
            Опкод::Call => ИнстрV3::Call(self.прочитать_u32()?),
            Опкод::Ret => ИнстрV3::Ret,
            Опкод::CallNative => ИнстрV3::CallNative {
                lib: self.прочитать_u32()?,
                fun: self.прочитать_u32()?,
            },
            Опкод::Pulse => ИнстрV3::Pulse {
                из: self.прочитать_u16()?,
                в: self.прочитать_u16()?,
            },
            Опкод::PerfectLock => ИнстрV3::PerfectLock(self.прочитать_u16()?),
            Опкод::ChoirSpawn => ИнстрV3::ChoirSpawn(self.прочитать_u32()?),
            Опкод::ChoirJoin => ИнстрV3::ChoirJoin,
            Опкод::AsyncCall => ИнстрV3::AsyncCall(self.прочитать_u32()?),
            Опкод::NewSet => ИнстрV3::NewSet,
            Опкод::NewIter => ИнстрV3::NewIter,
            Опкод::NewNothing => ИнстрV3::NewNothing,
            Опкод::NewSomething => ИнстрV3::NewSomething,
            Опкод::NewSuccess => ИнстрV3::NewSuccess,
            Опкод::NewError => ИнстрV3::NewError,
            Опкод::CheckTag => ИнстрV3::CheckTag,
            Опкод::Print => ИнстрV3::Print,
            Опкод::PrintLn => ИнстрV3::PrintLn,
            Опкод::PrintI64 => ИнстрV3::PrintI64,
        })
    }

    fn прочитать_u8(&mut self) -> Result<u8, ОшибкаДекодирования> {
        if self.позиция >= self.данные.len() {
            return Err(ОшибкаДекодирования::КонецДанных);
        }
        let b = self.данные[self.позиция];
        self.позиция += 1;
        Ok(b)
    }

    fn прочитать_u16(&mut self) -> Result<u16, ОшибкаДекодирования> {
        let b = self.прочитать_байты(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn прочитать_u32(&mut self) -> Result<u32, ОшибкаДекодирования> {
        let b = self.прочитать_байты(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn прочитать_i64(&mut self) -> Result<i64, ОшибкаДекодирования> {
        let b = self.прочитать_байты(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(i64::from_le_bytes(a))
    }

    fn прочитать_f64(&mut self) -> Result<f64, ОшибкаДекодирования> {
        let b = self.прочитать_байты(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(f64::from_le_bytes(a))
    }

    fn прочитать_байты(&mut self, n: usize) -> Result<&'a [u8], ОшибкаДекодирования> {
        if self.позиция + n > self.данные.len() {
            return Err(ОшибкаДекодирования::КонецДанных);
        }
        let s = &self.данные[self.позиция..self.позиция + n];
        self.позиция += n;
        Ok(s)
    }
}

/// Разобрать `.zipb` в МодульV3.
pub fn разобрать_v3(данные: &[u8]) -> Result<МодульV3, ОшибкаДекодирования> {
    Декодировщик::новый(данные).прочитать_модуль()
}

/// Декодировать тело функции в список инструкций.
pub fn декодировать_код(данные: &[u8]) -> Result<Vec<ИнстрV3>, ОшибкаДекодирования> {
    Декодировщик::новый(данные).декодировать_инструкции()
}

/// Конвертация v3 → внутренний модуль VM (для исполнителя).
pub fn в_модуль_vm(м: &МодульV3) -> Result<Модуль, ОшибкаДекодирования> {
    let mut функции = Vec::new();
    for ф in &м.функции {
        let имя = м
            .строки
            .get(ф.имя_индекс as usize)
            .cloned()
            .unwrap_or_else(|| format!("fn_{}", ф.имя_индекс));
        let instrs = декодировать_в_vm_инструкции(&ф.код)?;
        функции.push(Функция {
            имя,
            параметры: ф.арность.min(255) as u8,
            локальные: ф.локалы,
            размер_стека: 256,
            порог_горячности: 1000,
            код: instrs,
        });
    }
    Ok(Модуль {
        версия: ВЕРСИЯ_VM,
        флаги: м.флаги,
        строки: м.строки.clone(),
        функции,
    })
}

fn декодировать_в_vm_инструкции(код: &[u8]) -> Result<Vec<Инструкция>, ОшибкаДекодирования> {
    let mut d = Декодировщик::новый(код);
    let mut out = Vec::new();
    while d.позиция < d.данные.len() {
        let i = d.прочитать_инструкцию()?;
        out.push(match i {
            ИнстрV3::Nop => Инструкция::Nop,
            ИнстрV3::PushI64(n) => Инструкция::PushI64(n),
            ИнстрV3::PushF64(x) => Инструкция::PushF64(x),
            ИнстрV3::PushStr(idx) => Инструкция::PushI64(idx as i64),
            ИнстрV3::PushNull | ИнстрV3::PushFalse => Инструкция::PushI64(0),
            ИнстрV3::PushTrue => Инструкция::PushI64(1),
            ИнстрV3::Add => Инструкция::Add,
            ИнстрV3::Sub => Инструкция::Sub,
            ИнстрV3::Mul => Инструкция::Mul,
            ИнстрV3::Div => Инструкция::Div,
            ИнстрV3::Mod => Инструкция::Div, // MVP: mod→div placeholder not ideal; keep Div
            ИнстрV3::Eq => Инструкция::CmpEq,
            ИнстрV3::Ne => Инструкция::CmpNe,
            ИнстрV3::Lt => Инструкция::CmpLt,
            ИнстрV3::Le => Инструкция::CmpLe,
            ИнстрV3::Gt => Инструкция::CmpGt,
            ИнстрV3::Ge => Инструкция::CmpGe,
            ИнстрV3::Load(r) => Инструкция::Load(r.min(u16::MAX as u32) as u16),
            ИнстрV3::Store(r) => Инструкция::Store(r.min(u16::MAX as u32) as u16),
            ИнстрV3::Jump(a) => Инструкция::Jump(a),
            ИнстрV3::JumpIf(a) => Инструкция::JumpIf(a),
            ИнстрV3::JumpIfNot(a) => Инструкция::JumpIfNot(a),
            ИнстрV3::Call(f) => Инструкция::Call(f),
            ИнстрV3::Ret => Инструкция::Ret,
            ИнстрV3::Pulse { из, в } => Инструкция::Pulse { из, в },
            ИнстрV3::PerfectLock(r) => Инструкция::PerfectLock(r),
            ИнстрV3::ChoirSpawn(f) => Инструкция::ChoirSpawn(f),
            ИнстрV3::ChoirJoin | ИнстрV3::AsyncCall(_) => Инструкция::ChoirJoin,
            ИнстрV3::Print => Инструкция::Print,
            ИнстрV3::PrintLn => Инструкция::PrintLn,
            ИнстрV3::PrintI64 => Инструкция::PrintI64,
            ИнстрV3::Pop | ИнстрV3::Dup | ИнстрV3::Swap => Инструкция::Nop,
            ИнстрV3::CallNative { .. }
            | ИнстрV3::NewSet
            | ИнстрV3::NewIter
            | ИнстрV3::NewNothing
            | ИнстрV3::NewSomething
            | ИнстрV3::NewSuccess
            | ИнстрV3::NewError
            | ИнстрV3::CheckTag => Инструкция::Nop,
        });
    }
    Ok(out)
}

/// Являются ли байты модулем ZIPB v3?
pub fn это_zipb(данные: &[u8]) -> bool {
    данные.len() >= 4 && &данные[0..4] == МАГИЯ_V3
}
