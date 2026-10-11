//! Пул памяти (bump-arena) для снижения аллокаций при разборе AST.
//!
//! Один или несколько блоков по 64 КБ; освобождение — только `очистить()`.

use std::cell::RefCell;

const РАЗМЕР_БЛОКА: usize = 64 * 1024;

/// bump-allocator для временных буферов парсера/IR.
pub struct ПулПамяти {
    блоки: RefCell<Vec<Box<[u8]>>>,
    /// (индекс блока, смещение)
    текущий: RefCell<(usize, usize)>,
    использовано_всего: RefCell<usize>,
}

impl ПулПамяти {
    pub fn новый() -> Self {
        Self {
            блоки: RefCell::new(Vec::new()),
            текущий: RefCell::new((0, 0)),
            использовано_всего: RefCell::new(0),
        }
    }

    /// Выделить `размер` байт (выравнивание 8).
    pub fn выделить(&self, размер: usize) -> *mut u8 {
        let размер = размер.max(1).saturating_add(7) & !7;
        if размер > РАЗМЕР_БЛОКА {
            // Крупный блок под один запрос
            let mut блок = vec![0u8; размер].into_boxed_slice();
            let ptr = блок.as_mut_ptr();
            self.блоки.borrow_mut().push(блок);
            *self.использовано_всего.borrow_mut() += размер;
            return ptr;
        }

        loop {
            let (индекс, смещение) = *self.текущий.borrow();
            let mut блоки = self.блоки.borrow_mut();
            if индекс >= блоки.len() {
                блоки.push(vec![0u8; РАЗМЕР_БЛОКА].into_boxed_slice());
                *self.текущий.borrow_mut() = (блоки.len() - 1, 0);
                continue;
            }
            if смещение + размер > РАЗМЕР_БЛОКА {
                блоки.push(vec![0u8; РАЗМЕР_БЛОКА].into_boxed_slice());
                *self.текущий.borrow_mut() = (блоки.len() - 1, 0);
                continue;
            }
            let ptr = unsafe { блоки[индекс].as_mut_ptr().add(смещение) };
            *self.текущий.borrow_mut() = (индекс, смещение + размер);
            *self.использовано_всего.borrow_mut() += размер;
            return ptr;
        }
    }

    /// Выделить и вернуть срез (живёт, пока пул не очищен).
    ///
    /// # Safety
    /// Срез валиден до `очистить()` / drop пула.
    pub unsafe fn выделить_срез(&self, размер: usize) -> &'static mut [u8] {
        let ptr = self.выделить(размер);
        std::slice::from_raw_parts_mut(ptr, размер)
    }

    pub fn очистить(&self) {
        self.блоки.borrow_mut().clear();
        *self.текущий.borrow_mut() = (0, 0);
        *self.использовано_всего.borrow_mut() = 0;
    }

    /// (ёмкость всех блоков, использовано байт)
    pub fn статистика(&self) -> (usize, usize) {
        let блоки = self.блоки.borrow();
        let всего = блоки.iter().map(|б| б.len()).sum();
        let использовано = *self.использовано_всего.borrow();
        (всего, использовано)
    }
}

impl Default for ПулПамяти {
    fn default() -> Self {
        Self::новый()
    }
}

// Пул не Sync из-за RefCell — намеренно однопоточный на этапе парсинга.
unsafe impl Send for ПулПамяти {}
