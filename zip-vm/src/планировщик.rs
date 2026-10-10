//! Планировщик импульсов и хоров (легковесные задачи).

use crate::байткод::{Значение, ОшибкаVM};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct ЗадачаХора {
    pub функция: u32,
    pub аргумент: Значение,
    pub результат: Option<Значение>,
    pub готова: bool,
}

#[derive(Debug, Default)]
pub struct Планировщик {
    очередь: VecDeque<ЗадачаХора>,
    активные: Vec<Arc<Mutex<ЗадачаХора>>>,
    импульсы_ожидания: Vec<(u16, bool)>,
}

impl Планировщик {
    pub fn новый() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, функция: u32, аргумент: Значение) -> usize {
        let задача = Arc::new(Mutex::new(ЗадачаХора {
            функция,
            аргумент: аргумент.clone(),
            результат: Some(аргумент),
            готова: false,
        }));
        let клон = Arc::clone(&задача);
        // Упрощённый хор: фоновый поток сразу помечает задачу готовой
        // (полноценный JIT-вызов функции — следующий этап).
        thread::spawn(move || {
            if let Ok(mut з) = клон.lock() {
                з.готова = true;
            }
        });
        let id = self.активные.len();
        self.активные.push(задача);
        id
    }

    pub fn join_все(&mut self) -> Result<Vec<Значение>, ОшибкаVM> {
        let mut результаты = Vec::new();
        for задача in &self.активные {
            loop {
                let готова = задача.lock().map(|з| з.готова).unwrap_or(true);
                if готова {
                    break;
                }
                thread::yield_now();
            }
            if let Ok(з) = задача.lock() {
                результаты.push(з.результат.clone().unwrap_or(Значение::Пусто));
            }
        }
        self.активные.clear();
        Ok(результаты)
    }

    pub fn pulse_wait(&mut self, регистр: u16) {
        self.импульсы_ожидания.push((регистр, true));
    }

    pub fn импульс_готов(&self, регистр: u16) -> bool {
        self.импульсы_ожидания
            .iter()
            .any(|(р, готов)| *р == регистр && *готов)
    }

    pub fn поставить_в_очередь(&mut self, задача: ЗадачаХора) {
        self.очередь.push_back(задача);
    }

    pub fn взять_из_очереди(&mut self) -> Option<ЗадачаХора> {
        self.очередь.pop_front()
    }
}
