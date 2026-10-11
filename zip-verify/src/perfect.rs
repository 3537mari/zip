//! Верификация PERFECT-инварианта.

use std::collections::HashSet;
use zip_vm::байткод_v3::{ИнстрV3, ФункцияV3};
use zip_vm::байткод_v3_декодер::декодировать_код;

#[derive(Debug)]
pub enum ОшибкаPerfect {
    Модификация(usize, u16),
    Декод(String),
}

impl std::fmt::Display for ОшибкаPerfect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Модификация(i, r) => {
                write!(f, "Store в Совершенный регистр {r} (инстр. {i})")
            }
            Self::Декод(s) => write!(f, "декодирование: {s}"),
        }
    }
}

/// После `PerfectLock(r)` запрещён `Store(r)` (содержимое неизменяемо).
pub fn проверить_инструкции(инструкции: &[ИнстрV3]) -> Result<(), ОшибкаPerfect> {
    let mut совершенные: HashSet<u16> = HashSet::new();

    for (индекс, инструкция) in инструкции.iter().enumerate() {
        match инструкция {
            ИнстрV3::PerfectLock(р) => {
                совершенные.insert(*р);
            }
            ИнстрV3::Store(регистр) => {
                let р = *регистр as u16;
                if совершенные.contains(&р) {
                    return Err(ОшибкаPerfect::Модификация(индекс, р));
                }
            }
            ИнстрV3::Pulse { из, в } => {
                // Move оболочки: совершенность переезжает вместе со значением.
                let был = совершенные.remove(из);
                if был {
                    совершенные.insert(*в);
                } else {
                    совершенные.remove(в);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn проверить_perfect(функция: &ФункцияV3) -> Result<(), ОшибкаPerfect> {
    let инструкции = декодировать_код(&функция.код)
        .map_err(|е| ОшибкаPerfect::Декод(е.to_string()))?;
    проверить_инструкции(&инструкции)
}
