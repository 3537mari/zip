//! Стек значений и кадры вызовов.

use crate::байткод::{Значение, ОшибкаVM};

#[derive(Debug, Clone)]
pub struct Кадр {
    pub функция: u32,
    pub возврат_ip: usize,
    pub база_локальных: usize,
    pub база_стека: usize,
    /// Регистры вызывающей функции (восстанавливаются при Ret).
    pub сохранённые_регистры: Vec<Значение>,
}

#[derive(Debug, Default)]
pub struct СтекВызовов {
    кадры: Vec<Кадр>,
}

impl СтекВызовов {
    pub fn новый() -> Self {
        Self { кадры: Vec::new() }
    }

    pub fn войти(&mut self, кадр: Кадр) {
        self.кадры.push(кадр);
    }

    pub fn выйти(&mut self) -> Result<Кадр, ОшибкаVM> {
        self.кадры
            .pop()
            .ok_or_else(|| ОшибкаVM::новый("стек вызовов пуст"))
    }

    pub fn текущий(&self) -> Option<&Кадр> {
        self.кадры.last()
    }

    pub fn глубина(&self) -> usize {
        self.кадры.len()
    }
}

#[derive(Debug, Default)]
pub struct СтекЗначений {
    данные: Vec<Значение>,
}

impl СтекЗначений {
    pub fn новый() -> Self {
        Self { данные: Vec::new() }
    }

    pub fn push(&mut self, з: Значение) {
        self.данные.push(з);
    }

    pub fn pop(&mut self) -> Result<Значение, ОшибкаVM> {
        self.данные
            .pop()
            .ok_or_else(|| ОшибкаVM::новый("стек значений пуст"))
    }

    pub fn peek(&self) -> Result<&Значение, ОшибкаVM> {
        self.данные
            .last()
            .ok_or_else(|| ОшибкаVM::новый("стек значений пуст"))
    }

    pub fn обрезать(&mut self, до: usize) {
        self.данные.truncate(до);
    }

    pub fn длина(&self) -> usize {
        self.данные.len()
    }
}
