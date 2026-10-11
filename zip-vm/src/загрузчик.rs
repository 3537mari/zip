//! Загрузка модуля .zipc с диска.

use crate::байткод::{
    опкоды, Инструкция, Модуль, Функция, ОшибкаVM, ВЕРСИЯ, МАГИЯ,
};
use std::fs;
use std::path::Path;

pub fn загрузить(путь: &Path) -> Result<Модуль, ОшибкаVM> {
    let байты = fs::read(путь).map_err(|е| ОшибкаVM::новый(е.to_string()))?;
    разобрать(&байты)
}

pub fn разобрать(данные: &[u8]) -> Result<Модуль, ОшибкаVM> {
    let mut r = Читатель::новый(данные);

    let магия = r.байты(4)?;
    if магия != МАГИЯ {
        return Err(ОшибкаVM::новый("неверная магия (ожидалось ZIPC)"));
    }
    let версия = r.u16()?;
    if версия > ВЕРСИЯ {
        return Err(ОшибкаVM::новый(format!("неподдерживаемая версия {}", версия)));
    }
    let флаги = r.u16()?;
    let число_строк = r.u32()? as usize;
    let число_функций = r.u32()? as usize;

    let mut строки = Vec::with_capacity(число_строк);
    for _ in 0..число_строк {
        let len = r.u32()? as usize;
        let bytes = r.байты(len)?;
        let s = String::from_utf8(bytes.to_vec())
            .map_err(|е| ОшибкаVM::новый(е.to_string()))?;
        строки.push(s);
    }

    let mut функции = Vec::with_capacity(число_функций);
    for _ in 0..число_функций {
        let индекс_имени = r.u32()? as usize;
        let имя = строки
            .get(индекс_имени)
            .cloned()
            .unwrap_or_else(|| format!("fn_{}", индекс_имени));
        let параметры = r.u8()?;
        let локальные = r.u16()?;
        let размер_стека = r.u32()?;
        let порог_горячности = if версия >= 2 {
            r.u64()?
        } else {
            1000
        };
        let размер_кода = r.u32()? as usize;
        let код_байты = r.байты(размер_кода)?;
        let код = декодировать_инструкции(код_байты)?;
        функции.push(Функция {
            имя,
            параметры,
            локальные,
            размер_стека,
            порог_горячности,
            код,
        });
    }

    Ok(Модуль {
        версия,
        флаги,
        строки,
        функции,
    })
}

fn декодировать_инструкции(данные: &[u8]) -> Result<Vec<Инструкция>, ОшибкаVM> {
    let mut r = Читатель::новый(данные);
    let mut out = Vec::new();
    while r.осталось() > 0 {
        let op = r.u8()?;
        let инстр = match op {
            опкоды::NOP => Инструкция::Nop,
            опкоды::PUSH_I64 => Инструкция::PushI64(r.i64()?),
            опкоды::PUSH_F64 => Инструкция::PushF64(f64::from_bits(r.u64()?)),
            опкоды::ADD => Инструкция::Add,
            опкоды::SUB => Инструкция::Sub,
            опкоды::MUL => Инструкция::Mul,
            опкоды::DIV => Инструкция::Div,
            опкоды::CMP_EQ => Инструкция::CmpEq,
            опкоды::CMP_NE => Инструкция::CmpNe,
            опкоды::CMP_LT => Инструкция::CmpLt,
            опкоды::CMP_GT => Инструкция::CmpGt,
            опкоды::CMP_LE => Инструкция::CmpLe,
            опкоды::CMP_GE => Инструкция::CmpGe,
            опкоды::PULSE => Инструкция::Pulse {
                из: r.u16()?,
                в: r.u16()?,
            },
            опкоды::PULSE_WAIT => Инструкция::PulseWait(r.u16()?),
            опкоды::PERFECT_LOCK => Инструкция::PerfectLock(r.u16()?),
            опкоды::CHOIR_SPAWN => Инструкция::ChoirSpawn(r.u32()?),
            опкоды::CHOIR_JOIN => Инструкция::ChoirJoin,
            опкоды::LOAD => Инструкция::Load(r.u16()?),
            опкоды::STORE => Инструкция::Store(r.u16()?),
            опкоды::JUMP => Инструкция::Jump(r.u32()?),
            опкоды::JUMP_IF => Инструкция::JumpIf(r.u32()?),
            опкоды::JUMP_IF_NOT => Инструкция::JumpIfNot(r.u32()?),
            опкоды::CALL => Инструкция::Call(r.u32()?),
            опкоды::RET => Инструкция::Ret,
            опкоды::PRINT => Инструкция::Print,
            опкоды::PRINT_LN => Инструкция::PrintLn,
            опкоды::PRINT_I64 => Инструкция::PrintI64,
            другой => {
                return Err(ОшибкаVM::новый(format!("неизвестный опкод 0x{:02x}", другой)))
            }
        };
        out.push(инстр);
    }
    Ok(out)
}

struct Читатель<'a> {
    данные: &'a [u8],
    поз: usize,
}

impl<'a> Читатель<'a> {
    fn новый(данные: &'a [u8]) -> Self {
        Self { данные, поз: 0 }
    }

    fn осталось(&self) -> usize {
        self.данные.len().saturating_sub(self.поз)
    }

    fn байты(&mut self, n: usize) -> Result<&'a [u8], ОшибкаVM> {
        if self.поз + n > self.данные.len() {
            return Err(ОшибкаVM::новый("неожиданный конец файла"));
        }
        let s = &self.данные[self.поз..self.поз + n];
        self.поз += n;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, ОшибкаVM> {
        Ok(self.байты(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ОшибкаVM> {
        let b = self.байты(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, ОшибкаVM> {
        let b = self.байты(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, ОшибкаVM> {
        let b = self.байты(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(u64::from_le_bytes(a))
    }

    fn i64(&mut self) -> Result<i64, ОшибкаVM> {
        Ok(self.u64()? as i64)
    }
}

/// Сериализация модуля в байты .zipc (для тестов и zipc).
pub fn сериализовать(модуль: &Модуль) -> Result<Vec<u8>, ОшибкаVM> {
    let mut out = Vec::new();
    out.extend_from_slice(МАГИЯ);
    out.extend_from_slice(&модуль.версия.to_le_bytes());
    out.extend_from_slice(&модуль.флаги.to_le_bytes());
    out.extend_from_slice(&(модуль.строки.len() as u32).to_le_bytes());
    out.extend_from_slice(&(модуль.функции.len() as u32).to_le_bytes());

    for s in &модуль.строки {
        let b = s.as_bytes();
        out.extend_from_slice(&(b.len() as u32).to_le_bytes());
        out.extend_from_slice(b);
    }

    for ф in &модуль.функции {
        let idx = модуль
            .строки
            .iter()
            .position(|s| s == &ф.имя)
            .unwrap_or(0) as u32;
        out.extend_from_slice(&idx.to_le_bytes());
        out.push(ф.параметры);
        out.extend_from_slice(&ф.локальные.to_le_bytes());
        out.extend_from_slice(&ф.размер_стека.to_le_bytes());
        out.extend_from_slice(&ф.порог_горячности.to_le_bytes());
        let код = кодировать_инструкции(&ф.код);
        out.extend_from_slice(&(код.len() as u32).to_le_bytes());
        out.extend_from_slice(&код);
    }
    Ok(out)
}

fn кодировать_инструкции(код: &[Инструкция]) -> Vec<u8> {
    let mut out = Vec::new();
    for и in код {
        match и {
            Инструкция::Nop => out.push(опкоды::NOP),
            Инструкция::PushI64(n) => {
                out.push(опкоды::PUSH_I64);
                out.extend_from_slice(&n.to_le_bytes());
            }
            Инструкция::PushF64(x) => {
                out.push(опкоды::PUSH_F64);
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
            Инструкция::Add => out.push(опкоды::ADD),
            Инструкция::Sub => out.push(опкоды::SUB),
            Инструкция::Mul => out.push(опкоды::MUL),
            Инструкция::Div => out.push(опкоды::DIV),
            Инструкция::CmpEq => out.push(опкоды::CMP_EQ),
            Инструкция::CmpNe => out.push(опкоды::CMP_NE),
            Инструкция::CmpLt => out.push(опкоды::CMP_LT),
            Инструкция::CmpGt => out.push(опкоды::CMP_GT),
            Инструкция::CmpLe => out.push(опкоды::CMP_LE),
            Инструкция::CmpGe => out.push(опкоды::CMP_GE),
            Инструкция::Pulse { из, в } => {
                out.push(опкоды::PULSE);
                out.extend_from_slice(&из.to_le_bytes());
                out.extend_from_slice(&в.to_le_bytes());
            }
            Инструкция::PulseWait(r) => {
                out.push(опкоды::PULSE_WAIT);
                out.extend_from_slice(&r.to_le_bytes());
            }
            Инструкция::PerfectLock(r) => {
                out.push(опкоды::PERFECT_LOCK);
                out.extend_from_slice(&r.to_le_bytes());
            }
            Инструкция::ChoirSpawn(f) => {
                out.push(опкоды::CHOIR_SPAWN);
                out.extend_from_slice(&f.to_le_bytes());
            }
            Инструкция::ChoirJoin => out.push(опкоды::CHOIR_JOIN),
            Инструкция::Load(r) => {
                out.push(опкоды::LOAD);
                out.extend_from_slice(&r.to_le_bytes());
            }
            Инструкция::Store(r) => {
                out.push(опкоды::STORE);
                out.extend_from_slice(&r.to_le_bytes());
            }
            Инструкция::Jump(a) => {
                out.push(опкоды::JUMP);
                out.extend_from_slice(&a.to_le_bytes());
            }
            Инструкция::JumpIf(a) => {
                out.push(опкоды::JUMP_IF);
                out.extend_from_slice(&a.to_le_bytes());
            }
            Инструкция::JumpIfNot(a) => {
                out.push(опкоды::JUMP_IF_NOT);
                out.extend_from_slice(&a.to_le_bytes());
            }
            Инструкция::Call(f) => {
                out.push(опкоды::CALL);
                out.extend_from_slice(&f.to_le_bytes());
            }
            Инструкция::Ret => out.push(опкоды::RET),
            Инструкция::Print => out.push(опкоды::PRINT),
            Инструкция::PrintLn => out.push(опкоды::PRINT_LN),
            Инструкция::PrintI64 => out.push(опкоды::PRINT_I64),
        }
    }
    out
}
