//! Формат байт-кода ZIP (.zipc).

use std::fmt;

pub const МАГИЯ: &[u8; 4] = b"ZIPC";
pub const ВЕРСИЯ: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum Значение {
    Целое(i64),
    Вещественное(f64),
    Указатель(usize),
    Пусто,
}

impl Значение {
    pub fn как_i64(&self) -> Result<i64, ОшибкаVM> {
        match self {
            Значение::Целое(n) => Ok(*n),
            Значение::Вещественное(x) => Ok(*x as i64),
            Значение::Указатель(p) => Ok(*p as i64),
            Значение::Пусто => Err(ОшибкаVM::новый("пустое значение")),
        }
    }

    pub fn истинно(&self) -> bool {
        match self {
            Значение::Целое(n) => *n != 0,
            Значение::Вещественное(x) => *x != 0.0,
            Значение::Указатель(p) => *p != 0,
            Значение::Пусто => false,
        }
    }

    pub fn сложить(self, другое: Значение) -> Result<Значение, ОшибкаVM> {
        Ok(Значение::Целое(self.как_i64()? + другое.как_i64()?))
    }

    pub fn вычесть(self, другое: Значение) -> Result<Значение, ОшибкаVM> {
        Ok(Значение::Целое(self.как_i64()? - другое.как_i64()?))
    }

    pub fn умножить(self, другое: Значение) -> Result<Значение, ОшибкаVM> {
        Ok(Значение::Целое(self.как_i64()? * другое.как_i64()?))
    }

    pub fn разделить(self, другое: Значение) -> Result<Значение, ОшибкаVM> {
        let b = другое.как_i64()?;
        if b == 0 {
            return Err(ОшибкаVM::новый("деление на ноль"));
        }
        Ok(Значение::Целое(self.как_i64()? / b))
    }
}

impl fmt::Display for Значение {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Значение::Целое(n) => write!(f, "{}", n),
            Значение::Вещественное(x) => write!(f, "{}", x),
            Значение::Указатель(p) => write!(f, "ptr({})", p),
            Значение::Пусто => write!(f, "пусто"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Инструкция {
    Nop,
    PushI64(i64),
    PushF64(f64),
    Add,
    Sub,
    Mul,
    Div,
    CmpEq,
    CmpNe,
    CmpLt,
    CmpGt,
    CmpLe,
    CmpGe,
    Pulse { из: u16, в: u16 },
    PulseWait(u16),
    PerfectLock(u16),
    ChoirSpawn(u32),
    ChoirJoin,
    Load(u16),
    Store(u16),
    Jump(u32),
    JumpIf(u32),
    JumpIfNot(u32),
    Call(u32),
    Ret,
    Print,
    PrintLn,
    PrintI64,
}

#[derive(Debug, Clone)]
pub struct Функция {
    pub имя: String,
    pub параметры: u8,
    pub локальные: u16,
    pub размер_стека: u32,
    pub код: Vec<Инструкция>,
}

#[derive(Debug, Clone)]
pub struct Модуль {
    pub версия: u16,
    pub флаги: u16,
    pub строки: Vec<String>,
    pub функции: Vec<Функция>,
}

#[derive(Debug, Clone)]
pub struct ОшибкаVM {
    pub сообщение: String,
}

impl ОшибкаVM {
    pub fn новый(с: impl Into<String>) -> Self {
        Self {
            сообщение: с.into(),
        }
    }
}

impl fmt::Display for ОшибкаVM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ошибка VM: {}", self.сообщение)
    }
}

impl std::error::Error for ОшибкаVM {}

/// Оpcodes (совпадают со спецификацией).
pub mod опкоды {
    pub const NOP: u8 = 0x00;
    pub const PUSH_I64: u8 = 0x01;
    pub const PUSH_F64: u8 = 0x02;
    pub const ADD: u8 = 0x10;
    pub const SUB: u8 = 0x11;
    pub const MUL: u8 = 0x12;
    pub const DIV: u8 = 0x13;
    pub const CMP_EQ: u8 = 0x14;
    pub const CMP_NE: u8 = 0x15;
    pub const CMP_LT: u8 = 0x16;
    pub const CMP_GT: u8 = 0x17;
    pub const CMP_LE: u8 = 0x18;
    pub const CMP_GE: u8 = 0x19;
    pub const PULSE: u8 = 0x20;
    pub const PULSE_WAIT: u8 = 0x21;
    pub const PERFECT_LOCK: u8 = 0x22;
    pub const CHOIR_SPAWN: u8 = 0x23;
    pub const CHOIR_JOIN: u8 = 0x24;
    pub const LOAD: u8 = 0x30;
    pub const STORE: u8 = 0x31;
    pub const JUMP: u8 = 0x40;
    pub const JUMP_IF: u8 = 0x41;
    pub const JUMP_IF_NOT: u8 = 0x42;
    pub const CALL: u8 = 0x50;
    pub const RET: u8 = 0x51;
    pub const PRINT: u8 = 0x60;
    pub const PRINT_LN: u8 = 0x61;
    pub const PRINT_I64: u8 = 0x62;
}
