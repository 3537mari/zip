//! Байт-код ZIP v3 — собственный формат (магия ZIPB).

/// Магия файла `.zipb`.
pub const МАГИЯ_V3: &[u8; 4] = b"ZIPB";
/// Версия формата.
pub const ВЕРСИЯ_V3: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Опкод {
    Nop = 0x00,
    Pop = 0x01,
    Dup = 0x02,
    Swap = 0x03,

    PushI64 = 0x10,
    PushF64 = 0x11,
    PushStr = 0x12,
    PushNull = 0x13,
    PushTrue = 0x14,
    PushFalse = 0x15,

    Add = 0x20,
    Sub = 0x21,
    Mul = 0x22,
    Div = 0x23,
    Mod = 0x24,

    Eq = 0x30,
    Ne = 0x31,
    Lt = 0x32,
    Le = 0x33,
    Gt = 0x34,
    Ge = 0x35,

    Load = 0x40,
    Store = 0x41,

    Jump = 0x50,
    JumpIf = 0x51,
    JumpIfNot = 0x52,

    Call = 0x60,
    Ret = 0x61,
    CallNative = 0x62,

    Pulse = 0x70,
    PerfectLock = 0x71,
    ChoirSpawn = 0x72,
    ChoirJoin = 0x73,
    AsyncCall = 0x74,

    NewSet = 0x80,
    NewIter = 0x81,
    NewNothing = 0x82,
    NewSomething = 0x83,
    NewSuccess = 0x84,
    NewError = 0x85,
    CheckTag = 0x86,

    Print = 0x90,
    PrintLn = 0x91,
    PrintI64 = 0x92,

    NewStruct = 0x93,
    LoadField = 0x94,
    StoreField = 0x95,
    GetFieldPtr = 0x96,
}

impl Опкод {
    pub fn из_байта(b: u8) -> Option<Self> {
        Some(match b {
            0x00 => Self::Nop,
            0x01 => Self::Pop,
            0x02 => Self::Dup,
            0x03 => Self::Swap,
            0x10 => Self::PushI64,
            0x11 => Self::PushF64,
            0x12 => Self::PushStr,
            0x13 => Self::PushNull,
            0x14 => Self::PushTrue,
            0x15 => Self::PushFalse,
            0x20 => Self::Add,
            0x21 => Self::Sub,
            0x22 => Self::Mul,
            0x23 => Self::Div,
            0x24 => Self::Mod,
            0x30 => Self::Eq,
            0x31 => Self::Ne,
            0x32 => Self::Lt,
            0x33 => Self::Le,
            0x34 => Self::Gt,
            0x35 => Self::Ge,
            0x40 => Self::Load,
            0x41 => Self::Store,
            0x50 => Self::Jump,
            0x51 => Self::JumpIf,
            0x52 => Self::JumpIfNot,
            0x60 => Self::Call,
            0x61 => Self::Ret,
            0x62 => Self::CallNative,
            0x70 => Self::Pulse,
            0x71 => Self::PerfectLock,
            0x72 => Self::ChoirSpawn,
            0x73 => Self::ChoirJoin,
            0x74 => Self::AsyncCall,
            0x80 => Self::NewSet,
            0x81 => Self::NewIter,
            0x82 => Self::NewNothing,
            0x83 => Self::NewSomething,
            0x84 => Self::NewSuccess,
            0x85 => Self::NewError,
            0x86 => Self::CheckTag,
            0x90 => Self::Print,
            0x91 => Self::PrintLn,
            0x92 => Self::PrintI64,
            0x93 => Self::NewStruct,
            0x94 => Self::LoadField,
            0x95 => Self::StoreField,
            0x96 => Self::GetFieldPtr,
            _ => return None,
        })
    }
}

/// Низкоуровневый кодировщик байт-потока.
pub struct Кодировщик {
    pub буфер: Vec<u8>,
}

impl Кодировщик {
    pub fn новый() -> Self {
        Self {
            буфер: Vec::with_capacity(65536),
        }
    }

    pub fn написать_заголовок(&mut self, количество_функций: u32, смещение_строк: u32) {
        self.буфер.extend_from_slice(МАГИЯ_V3);
        self.буфер.extend_from_slice(&ВЕРСИЯ_V3.to_le_bytes());
        self.буфер.extend_from_slice(&0u16.to_le_bytes());
        self.буфер.extend_from_slice(&количество_функций.to_le_bytes());
        self.буфер.extend_from_slice(&смещение_строк.to_le_bytes());
    }

    pub fn написать_опкод(&mut self, опкод: Опкод) {
        self.буфер.push(опкод as u8);
    }

    pub fn написать_i64(&mut self, значение: i64) {
        self.буфер.extend_from_slice(&значение.to_le_bytes());
    }

    pub fn написать_f64(&mut self, значение: f64) {
        self.буфер.extend_from_slice(&значение.to_le_bytes());
    }

    pub fn написать_u32(&mut self, значение: u32) {
        self.буфер.extend_from_slice(&значение.to_le_bytes());
    }

    pub fn написать_u16(&mut self, значение: u16) {
        self.буфер.extend_from_slice(&значение.to_le_bytes());
    }

    pub fn написать_строку_данных(&mut self, строка: &str) {
        let байты = строка.as_bytes();
        self.написать_u32(байты.len() as u32);
        self.буфер.extend_from_slice(байты);
    }

    pub fn завершить(self) -> Vec<u8> {
        self.буфер
    }
}

/// Инструкция v3 с операндами (для тестов и дампа).
#[derive(Debug, Clone, PartialEq)]
pub enum ИнстрV3 {
    Nop,
    Pop,
    Dup,
    Swap,
    PushI64(i64),
    PushF64(f64),
    PushStr(u32),
    PushNull,
    PushTrue,
    PushFalse,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Load(u32),
    Store(u32),
    Jump(u32),
    JumpIf(u32),
    JumpIfNot(u32),
    Call(u32),
    Ret,
    CallNative { lib: u32, fun: u32 },
    Pulse { из: u16, в: u16 },
    PerfectLock(u16),
    ChoirSpawn(u32),
    ChoirJoin,
    AsyncCall(u32),
    NewSet,
    NewIter,
    NewNothing,
    NewSomething,
    NewSuccess,
    NewError,
    CheckTag,
    Print,
    PrintLn,
    PrintI64,
    NewStruct(u32),
    LoadField(u32),
    StoreField(u32),
    GetFieldPtr(u32),
}

impl ИнстрV3 {
    pub fn закодировать(&self, out: &mut Vec<u8>) {
        use ИнстрV3::*;
        match self {
            Nop => out.push(Опкод::Nop as u8),
            Pop => out.push(Опкод::Pop as u8),
            Dup => out.push(Опкод::Dup as u8),
            Swap => out.push(Опкод::Swap as u8),
            PushI64(n) => {
                out.push(Опкод::PushI64 as u8);
                out.extend_from_slice(&n.to_le_bytes());
            }
            PushF64(x) => {
                out.push(Опкод::PushF64 as u8);
                out.extend_from_slice(&x.to_le_bytes());
            }
            PushStr(i) => {
                out.push(Опкод::PushStr as u8);
                out.extend_from_slice(&i.to_le_bytes());
            }
            PushNull => out.push(Опкод::PushNull as u8),
            PushTrue => out.push(Опкод::PushTrue as u8),
            PushFalse => out.push(Опкод::PushFalse as u8),
            Add => out.push(Опкод::Add as u8),
            Sub => out.push(Опкод::Sub as u8),
            Mul => out.push(Опкод::Mul as u8),
            Div => out.push(Опкод::Div as u8),
            Mod => out.push(Опкод::Mod as u8),
            Eq => out.push(Опкод::Eq as u8),
            Ne => out.push(Опкод::Ne as u8),
            Lt => out.push(Опкод::Lt as u8),
            Le => out.push(Опкод::Le as u8),
            Gt => out.push(Опкод::Gt as u8),
            Ge => out.push(Опкод::Ge as u8),
            Load(r) => {
                out.push(Опкод::Load as u8);
                out.extend_from_slice(&r.to_le_bytes());
            }
            Store(r) => {
                out.push(Опкод::Store as u8);
                out.extend_from_slice(&r.to_le_bytes());
            }
            Jump(a) => {
                out.push(Опкод::Jump as u8);
                out.extend_from_slice(&a.to_le_bytes());
            }
            JumpIf(a) => {
                out.push(Опкод::JumpIf as u8);
                out.extend_from_slice(&a.to_le_bytes());
            }
            JumpIfNot(a) => {
                out.push(Опкод::JumpIfNot as u8);
                out.extend_from_slice(&a.to_le_bytes());
            }
            Call(f) => {
                out.push(Опкод::Call as u8);
                out.extend_from_slice(&f.to_le_bytes());
            }
            Ret => out.push(Опкод::Ret as u8),
            CallNative { lib, fun } => {
                out.push(Опкод::CallNative as u8);
                out.extend_from_slice(&lib.to_le_bytes());
                out.extend_from_slice(&fun.to_le_bytes());
            }
            Pulse { из, в } => {
                out.push(Опкод::Pulse as u8);
                out.extend_from_slice(&из.to_le_bytes());
                out.extend_from_slice(&в.to_le_bytes());
            }
            PerfectLock(r) => {
                out.push(Опкод::PerfectLock as u8);
                out.extend_from_slice(&r.to_le_bytes());
            }
            ChoirSpawn(f) => {
                out.push(Опкод::ChoirSpawn as u8);
                out.extend_from_slice(&f.to_le_bytes());
            }
            ChoirJoin => out.push(Опкод::ChoirJoin as u8),
            AsyncCall(f) => {
                out.push(Опкод::AsyncCall as u8);
                out.extend_from_slice(&f.to_le_bytes());
            }
            NewSet => out.push(Опкод::NewSet as u8),
            NewIter => out.push(Опкод::NewIter as u8),
            NewNothing => out.push(Опкод::NewNothing as u8),
            NewSomething => out.push(Опкод::NewSomething as u8),
            NewSuccess => out.push(Опкод::NewSuccess as u8),
            NewError => out.push(Опкод::NewError as u8),
            CheckTag => out.push(Опкод::CheckTag as u8),
            Print => out.push(Опкод::Print as u8),
            PrintLn => out.push(Опкод::PrintLn as u8),
            PrintI64 => out.push(Опкод::PrintI64 as u8),
            NewStruct(n) => {
                out.push(Опкод::NewStruct as u8);
                out.extend_from_slice(&n.to_le_bytes());
            }
            LoadField(о) => {
                out.push(Опкод::LoadField as u8);
                out.extend_from_slice(&о.to_le_bytes());
            }
            StoreField(о) => {
                out.push(Опкод::StoreField as u8);
                out.extend_from_slice(&о.to_le_bytes());
            }
            GetFieldPtr(о) => {
                out.push(Опкод::GetFieldPtr as u8);
                out.extend_from_slice(&о.to_le_bytes());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ФункцияV3 {
    pub имя_индекс: u32,
    pub арность: u16,
    pub локалы: u16,
    pub флаги: u32,
    pub код: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct МодульV3 {
    pub флаги: u16,
    pub строки: Vec<String>,
    pub функции: Vec<ФункцияV3>,
}

/// Собрать полный `.zipb` из модуля.
pub fn сериализовать_v3(модуль: &МодульV3) -> Vec<u8> {
    let mut к = Кодировщик::новый();
    к.написать_заголовок(модуль.функции.len() as u32, 16);

    к.написать_u32(модуль.строки.len() as u32);
    for s in &модуль.строки {
        к.написать_строку_данных(s);
    }

    for ф in &модуль.функции {
        к.написать_u32(ф.имя_индекс);
        к.написать_u16(ф.арность);
        к.написать_u16(ф.локалы);
        к.написать_u32(ф.флаги);
        к.написать_u32(ф.код.len() as u32);
        к.буфер.extend_from_slice(&ф.код);
    }
    к.завершить()
}
