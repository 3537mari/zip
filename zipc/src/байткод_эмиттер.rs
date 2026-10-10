//! Эмиттер байт-кода .zipc из промежуточного представления.

use crate::промежуток::{
    ИдентификаторФункции, Инструкция as Ир, РежимПечати, ФункцияПП,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const МАГИЯ: &[u8; 4] = b"ZIPC";
const ВЕРСИЯ: u16 = 1;

mod оп {
    pub const PUSH_I64: u8 = 0x01;
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
    pub const CALL: u8 = 0x50;
    pub const RET: u8 = 0x51;
    pub const PRINT: u8 = 0x60;
    pub const PRINT_LN: u8 = 0x61;
    pub const PRINT_I64: u8 = 0x62;
    pub const NOP: u8 = 0x00;
}

#[derive(Clone)]
enum VmOp {
    PushI64(i64),
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
    Pulse(u16, u16),
    PulseWait(u16),
    Perfect(u16),
    ChoirSpawn(u32),
    ChoirJoin,
    Load(u16),
    Store(u16),
    Call(u32),
    Ret,
    Print,
    PrintLn,
    PrintI64,
    Nop,
    /// Псевдо: метка (не попадает в код)
    Label(u32),
    JumpToLabel(u32),
    JumpIfToLabel(u32),
}

fn add_str(s: &str, строки: &mut Vec<String>) -> u32 {
    if let Some(i) = строки.iter().position(|x| x == s) {
        return i as u32;
    }
    строки.push(s.to_string());
    (строки.len() - 1) as u32
}

/// Скомпилировать функции ПП в файл .zipc.
pub fn записать_zipc(функции: &[ФункцияПП], путь: &Path) -> Result<(), String> {
    fs::write(путь, сгенерировать(функции)?).map_err(|е| е.to_string())
}

pub fn сгенерировать(функции: &[ФункцияПП]) -> Result<Vec<u8>, String> {
    let mut строки: Vec<String> = Vec::new();
    for ф in функции {
        add_str(&ф.имя, &mut строки);
    }

    let fn_index = |id: ИдентификаторФункции| -> u32 {
        функции
            .iter()
            .position(|f| f.идентификатор == id)
            .unwrap_or(0) as u32
    };

    let mut тела: Vec<(u32, u8, u16, Vec<u8>)> = Vec::new();
    for ф in функции {
        let mut ops = Vec::new();
        for и in &ф.тело {
            match и {
                Ир::Метка { идентификатор } => ops.push(VmOp::Label(идентификатор.0)),
                Ир::КонстантаЦелое { назначение, значение } => {
                    ops.push(VmOp::PushI64(*значение));
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::КонстантаСтрока { назначение, значение } => {
                    let idx = add_str(значение, &mut строки) as i64;
                    ops.push(VmOp::PushI64(idx));
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::Переместить {
                    назначение,
                    источник,
                }
                | Ир::Импульс {
                    назначение,
                    источник,
                } => ops.push(VmOp::Pulse(источник.0 as u16, назначение.0 as u16)),
                Ир::Сложить { назначение, а, б } => {
                    ops.push(VmOp::Load(а.0 as u16));
                    ops.push(VmOp::Load(б.0 as u16));
                    ops.push(VmOp::Add);
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::Вычесть { назначение, а, б } => {
                    ops.push(VmOp::Load(а.0 as u16));
                    ops.push(VmOp::Load(б.0 as u16));
                    ops.push(VmOp::Sub);
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::Умножить { назначение, а, б } => {
                    ops.push(VmOp::Load(а.0 as u16));
                    ops.push(VmOp::Load(б.0 as u16));
                    ops.push(VmOp::Mul);
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::Разделить { назначение, а, б } => {
                    ops.push(VmOp::Load(а.0 as u16));
                    ops.push(VmOp::Load(б.0 as u16));
                    ops.push(VmOp::Div);
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::Сравнить {
                    оператор,
                    назначение,
                    а,
                    б,
                } => {
                    ops.push(VmOp::Load(а.0 as u16));
                    ops.push(VmOp::Load(б.0 as u16));
                    ops.push(match оператор.as_str() {
                        "==" => VmOp::CmpEq,
                        "!=" => VmOp::CmpNe,
                        "<" => VmOp::CmpLt,
                        ">" => VmOp::CmpGt,
                        "<=" => VmOp::CmpLe,
                        ">=" => VmOp::CmpGe,
                        _ => VmOp::CmpNe,
                    });
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                Ир::Печать { режим, значение } => {
                    ops.push(VmOp::Load(значение.0 as u16));
                    ops.push(match режим {
                        РежимПечати::СоСтрокой => VmOp::PrintLn,
                        РежимПечати::БезНовойСтроки => VmOp::Print,
                        РежимПечати::Число => VmOp::PrintI64,
                    });
                }
                Ир::ОжиданиеИмпульса { регистр } => {
                    ops.push(VmOp::PulseWait(регистр.0 as u16))
                }
                Ир::Зафиксировать { регистр } => ops.push(VmOp::Perfect(регистр.0 as u16)),
                Ир::ЗапуститьХор { функция, аргументы } => {
                    for арг in аргументы {
                        ops.push(VmOp::Load(арг.0 as u16));
                    }
                    if аргументы.is_empty() {
                        ops.push(VmOp::PushI64(0));
                    }
                    ops.push(VmOp::ChoirSpawn(fn_index(*функция)))
                }
                Ир::СобратьХор { .. } => ops.push(VmOp::ChoirJoin),
                Ир::Вызвать {
                    функция,
                    аргументы,
                    результат,
                } => {
                    for арг in аргументы {
                        ops.push(VmOp::Load(арг.0 as u16));
                    }
                    ops.push(VmOp::Call(fn_index(*функция)));
                    if let Some(р) = результат {
                        ops.push(VmOp::Store(р.0 as u16));
                    }
                }
                Ир::Перейти { метка } => ops.push(VmOp::JumpToLabel(метка.0)),
                Ир::ВетвлениеИмпульса {
                    условие,
                    истина,
                    ложь,
                } => {
                    ops.push(VmOp::Load(условие.0 as u16));
                    ops.push(VmOp::JumpIfToLabel(истина.0));
                    ops.push(VmOp::JumpToLabel(ложь.0));
                }
                Ир::Вернуть { значение } => {
                    if let Some(р) = значение {
                        ops.push(VmOp::Load(р.0 as u16));
                    } else {
                        ops.push(VmOp::PushI64(0));
                    }
                    ops.push(VmOp::Ret);
                }
                Ир::Выделить { назначение, .. }
                | Ир::КонстантаВещественное { назначение, .. } => {
                    ops.push(VmOp::PushI64(0));
                    ops.push(VmOp::Store(назначение.0 as u16));
                }
                _ => ops.push(VmOp::Nop),
            }
        }
        if !ops.iter().any(|o| matches!(o, VmOp::Ret)) {
            ops.push(VmOp::PushI64(0));
            ops.push(VmOp::Ret);
        }

        // Метки → индексы
        let mut label_at = HashMap::new();
        let mut idx = 0u32;
        for o in &ops {
            if let VmOp::Label(id) = o {
                label_at.insert(*id, idx);
            } else {
                idx += 1;
            }
        }

        let mut код = Vec::new();
        for o in ops {
            match o {
                VmOp::Label(_) => {}
                VmOp::PushI64(n) => {
                    код.push(оп::PUSH_I64);
                    код.extend_from_slice(&n.to_le_bytes());
                }
                VmOp::Add => код.push(оп::ADD),
                VmOp::Sub => код.push(оп::SUB),
                VmOp::Mul => код.push(оп::MUL),
                VmOp::Div => код.push(оп::DIV),
                VmOp::CmpEq => код.push(оп::CMP_EQ),
                VmOp::CmpNe => код.push(оп::CMP_NE),
                VmOp::CmpLt => код.push(оп::CMP_LT),
                VmOp::CmpGt => код.push(оп::CMP_GT),
                VmOp::CmpLe => код.push(оп::CMP_LE),
                VmOp::CmpGe => код.push(оп::CMP_GE),
                VmOp::Pulse(a, b) => {
                    код.push(оп::PULSE);
                    код.extend_from_slice(&a.to_le_bytes());
                    код.extend_from_slice(&b.to_le_bytes());
                }
                VmOp::PulseWait(r) => {
                    код.push(оп::PULSE_WAIT);
                    код.extend_from_slice(&r.to_le_bytes());
                }
                VmOp::Perfect(r) => {
                    код.push(оп::PERFECT_LOCK);
                    код.extend_from_slice(&r.to_le_bytes());
                }
                VmOp::ChoirSpawn(f) => {
                    код.push(оп::CHOIR_SPAWN);
                    код.extend_from_slice(&f.to_le_bytes());
                }
                VmOp::ChoirJoin => код.push(оп::CHOIR_JOIN),
                VmOp::Load(r) => {
                    код.push(оп::LOAD);
                    код.extend_from_slice(&r.to_le_bytes());
                }
                VmOp::Store(r) => {
                    код.push(оп::STORE);
                    код.extend_from_slice(&r.to_le_bytes());
                }
                VmOp::JumpToLabel(id) => {
                    код.push(оп::JUMP);
                    код.extend_from_slice(
                        &label_at.get(&id).copied().unwrap_or(0).to_le_bytes(),
                    );
                }
                VmOp::JumpIfToLabel(id) => {
                    код.push(оп::JUMP_IF);
                    код.extend_from_slice(
                        &label_at.get(&id).copied().unwrap_or(0).to_le_bytes(),
                    );
                }
                VmOp::Call(f) => {
                    код.push(оп::CALL);
                    код.extend_from_slice(&f.to_le_bytes());
                }
                VmOp::Ret => код.push(оп::RET),
                VmOp::Print => код.push(оп::PRINT),
                VmOp::PrintLn => код.push(оп::PRINT_LN),
                VmOp::PrintI64 => код.push(оп::PRINT_I64),
                VmOp::Nop => код.push(оп::NOP),
            }
        }

        let имя_idx = add_str(&ф.имя, &mut строки);
        let локальные = 64u16;
        тела.push((имя_idx, ф.параметры.len() as u8, локальные, код));
    }

    let mut out = Vec::new();
    out.extend_from_slice(МАГИЯ);
    out.extend_from_slice(&ВЕРСИЯ.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(строки.len() as u32).to_le_bytes());
    out.extend_from_slice(&(тела.len() as u32).to_le_bytes());
    for s in &строки {
        let b = s.as_bytes();
        out.extend_from_slice(&(b.len() as u32).to_le_bytes());
        out.extend_from_slice(b);
    }
    for (имя_idx, параметры, локальные, код) in тела {
        out.extend_from_slice(&имя_idx.to_le_bytes());
        out.push(параметры);
        out.extend_from_slice(&локальные.to_le_bytes());
        out.extend_from_slice(&64u32.to_le_bytes());
        out.extend_from_slice(&(код.len() as u32).to_le_bytes());
        out.extend_from_slice(&код);
    }
    Ok(out)
}
