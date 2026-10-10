//! Управление памятью VM.

use crate::байткод::{Значение, ОшибкаVM};

#[derive(Debug, Default)]
pub struct Память {
    данные: Vec<u8>,
    совершенные: Vec<bool>,
}

impl Память {
    pub fn новая(размер: usize) -> Self {
        Self {
            данные: vec![0; размер],
            совершенные: vec![false; размер / 8 + 1],
        }
    }

    pub fn выделить(&mut self, байт: usize) -> Result<usize, ОшибкаVM> {
        let адрес = self.данные.len();
        self.данные.resize(адрес + байт, 0);
        let слотов = (self.данные.len() + 7) / 8;
        self.совершенные.resize(слотов, false);
        Ok(адрес)
    }

    pub fn записать_i64(&mut self, адрес: usize, значение: i64) -> Result<(), ОшибкаVM> {
        let слот = адрес / 8;
        if self.совершенные.get(слот).copied().unwrap_or(false) {
            return Err(ОшибкаVM::новый("запись в PERFECT-память запрещена"));
        }
        if адрес + 8 > self.данные.len() {
            self.данные.resize(адрес + 8, 0);
        }
        self.данные[адрес..адрес + 8].copy_from_slice(&значение.to_le_bytes());
        Ok(())
    }

    pub fn читать_i64(&self, адрес: usize) -> Result<i64, ОшибкаVM> {
        if адрес + 8 > self.данные.len() {
            return Err(ОшибкаVM::новый("чтение за пределами памяти"));
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.данные[адрес..адрес + 8]);
        Ok(i64::from_le_bytes(buf))
    }

    pub fn perfect_lock(&mut self, адрес: usize) {
        let слот = адрес / 8;
        if слот >= self.совершенные.len() {
            self.совершенные.resize(слот + 1, false);
        }
        self.совершенные[слот] = true;
    }

    pub fn как_значение(&self, адрес: usize) -> Значение {
        Значение::Указатель(адрес)
    }
}
