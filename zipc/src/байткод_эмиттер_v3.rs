//! Эмиттер байт-кода ZIP v3 (.zipb) из промежуточного представления.

use crate::промежуток::{
    ИдентификаторФункции, Инструкция as Ир, РежимПечати, ФункцияПП,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use zip_vm::байткод_v3::{сериализовать_v3, ИнстрV3, МодульV3, ФункцияV3};

fn add_str(s: &str, строки: &mut Vec<String>) -> u32 {
    if let Some(i) = строки.iter().position(|x| x == s) {
        return i as u32;
    }
    строки.push(s.to_string());
    (строки.len() - 1) as u32
}

/// Записать `.zipb` на диск.
pub fn записать_zipb(функции: &[ФункцияПП], путь: &Path) -> Result<(), String> {
    fs::write(путь, сгенерировать_v3(функции)?).map_err(|е| е.to_string())
}

/// Сгенерировать байты ZIPB v3.
pub fn сгенерировать_v3(функции: &[ФункцияПП]) -> Result<Vec<u8>, String> {
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

    let mut функции_v3 = Vec::new();
    for ф in функции {
        let mut ops: Vec<Op> = Vec::new();
        for и in &ф.тело {
            match и {
                Ир::Метка { идентификатор } => ops.push(Op::Label(идентификатор.0)),
                Ир::КонстантаЦелое { назначение, значение } => {
                    ops.push(Op::I(ИнстрV3::PushI64(*значение)));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::КонстантаВещественное {
                    назначение,
                    значение,
                } => {
                    ops.push(Op::I(ИнстрV3::PushF64(*значение)));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::КонстантаСтрока {
                    назначение,
                    значение,
                } => {
                    let idx = add_str(значение, &mut строки);
                    ops.push(Op::I(ИнстрV3::PushStr(idx)));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Переместить {
                    назначение,
                    источник,
                }
                | Ир::Копировать {
                    назначение,
                    источник,
                } => {
                    ops.push(Op::I(ИнстрV3::Load(источник.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Импульс {
                    назначение,
                    источник,
                } => ops.push(Op::I(ИнстрV3::Pulse {
                    из: источник.0 as u16,
                    в: назначение.0 as u16,
                })),
                Ир::Сложить { назначение, а, б } => {
                    ops.push(Op::I(ИнстрV3::Load(а.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Load(б.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Add));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Вычесть { назначение, а, б } => {
                    ops.push(Op::I(ИнстрV3::Load(а.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Load(б.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Sub));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Умножить { назначение, а, б } => {
                    ops.push(Op::I(ИнстрV3::Load(а.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Load(б.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Mul));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Разделить { назначение, а, б } => {
                    ops.push(Op::I(ИнстрV3::Load(а.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Load(б.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Div));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Сравнить {
                    оператор,
                    назначение,
                    а,
                    б,
                } => {
                    ops.push(Op::I(ИнстрV3::Load(а.0 as u32)));
                    ops.push(Op::I(ИнстрV3::Load(б.0 as u32)));
                    ops.push(Op::I(match оператор.as_str() {
                        "==" => ИнстрV3::Eq,
                        "!=" => ИнстрV3::Ne,
                        "<" => ИнстрV3::Lt,
                        ">" => ИнстрV3::Gt,
                        "<=" => ИнстрV3::Le,
                        ">=" => ИнстрV3::Ge,
                        _ => ИнстрV3::Ne,
                    }));
                    ops.push(Op::I(ИнстрV3::Store(назначение.0 as u32)));
                }
                Ир::Печать { режим, значение } => {
                    ops.push(Op::I(ИнстрV3::Load(значение.0 as u32)));
                    ops.push(Op::I(match режим {
                        РежимПечати::СоСтрокой => ИнстрV3::PrintLn,
                        РежимПечати::БезНовойСтроки => ИнстрV3::Print,
                        РежимПечати::Число => ИнстрV3::PrintI64,
                    }));
                }
                Ир::Зафиксировать { регистр } => {
                    ops.push(Op::I(ИнстрV3::PerfectLock(регистр.0 as u16)))
                }
                Ир::ЗапуститьХор { функция, аргументы } => {
                    for арг in аргументы {
                        ops.push(Op::I(ИнстрV3::Load(арг.0 as u32)));
                    }
                    ops.push(Op::I(ИнстрV3::ChoirSpawn(fn_index(*функция))));
                }
                Ир::СобратьХор { .. } => ops.push(Op::I(ИнстрV3::ChoirJoin)),
                Ир::Вызвать {
                    функция,
                    аргументы,
                    результат,
                    ..
                } => {
                    for арг in аргументы {
                        ops.push(Op::I(ИнстрV3::Load(арг.0 as u32)));
                    }
                    ops.push(Op::I(ИнстрV3::Call(fn_index(*функция))));
                    if let Some(р) = результат {
                        ops.push(Op::I(ИнстрV3::Store(р.0 as u32)));
                    }
                }
                Ир::Перейти { метка } => ops.push(Op::JumpLabel(метка.0)),
                Ир::ВетвлениеИмпульса {
                    условие,
                    истина,
                    ложь,
                } => {
                    ops.push(Op::I(ИнстрV3::Load(условие.0 as u32)));
                    ops.push(Op::JumpIfLabel(истина.0));
                    ops.push(Op::JumpLabel(ложь.0));
                }
                Ир::Вернуть { значение } => {
                    if let Some(р) = значение {
                        ops.push(Op::I(ИнстрV3::Load(р.0 as u32)));
                    } else {
                        ops.push(Op::I(ИнстрV3::PushI64(0)));
                    }
                    ops.push(Op::I(ИнстрV3::Ret));
                }
                Ир::ВнешнийВызов {
                    библиотека,
                    функция,
                    ..
                } => {
                    let li = add_str(библиотека, &mut строки);
                    let fi = add_str(функция, &mut строки);
                    ops.push(Op::I(ИнстрV3::CallNative { lib: li, fun: fi }));
                }
                _ => ops.push(Op::I(ИнстрV3::Nop)),
            }
        }
        if !ops.iter().any(|o| matches!(o, Op::I(ИнстрV3::Ret))) {
            ops.push(Op::I(ИнстрV3::PushI64(0)));
            ops.push(Op::I(ИнстрV3::Ret));
        }

        let mut label_at = HashMap::new();
        let mut idx = 0u32;
        for o in &ops {
            match o {
                Op::Label(id) => {
                    label_at.insert(*id, idx);
                }
                _ => idx += 1,
            }
        }

        let mut код = Vec::new();
        for o in ops {
            match o {
                Op::Label(_) => {}
                Op::I(и) => и.закодировать(&mut код),
                Op::JumpLabel(id) => {
                    let a = *label_at.get(&id).unwrap_or(&0);
                    ИнстрV3::Jump(a).закодировать(&mut код);
                }
                Op::JumpIfLabel(id) => {
                    let a = *label_at.get(&id).unwrap_or(&0);
                    ИнстрV3::JumpIf(a).закодировать(&mut код);
                }
            }
        }

        let локалы = (ф.параметры.len() as u16 + 32).max(16);
        let флаги = (ф.асинхронная as u32)
            | ((ф.совершенная as u32) << 1)
            | ((ф.comptime as u32) << 2);
        функции_v3.push(ФункцияV3 {
            имя_индекс: add_str(&ф.имя, &mut строки),
            арность: ф.параметры.len() as u16,
            локалы,
            флаги,
            код,
        });
    }

    Ok(сериализовать_v3(&МодульV3 {
        флаги: 0,
        строки,
        функции: функции_v3,
    }))
}

enum Op {
    Label(u32),
    JumpLabel(u32),
    JumpIfLabel(u32),
    I(ИнстрV3),
}
