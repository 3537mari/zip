//! Тесты генератора нативного кода (≥ 10).

use zipc::нативный::{скомпилировать_в_ассемблер, ГенераторНативногоКода, ОпцииНативнойСборки};
use zipc::промежуток::сгенерировать_из_исходника;
use std::fs;
use std::path::PathBuf;

fn asm(исходник: &str) -> String {
    скомпилировать_в_ассемблер(исходник).unwrap_or_else(|е| panic!("{}", е))
}

#[test]
fn н01_заголовок_o3_native() {
    let т = asm("САПАР");
    assert!(т.contains("opt-level: O3"));
    assert!(т.contains("target-cpu: native"));
}

#[test]
fn н02_главная_метка() {
    let т = asm("САПАР");
    assert!(т.contains("zip_main:"));
    assert!(т.contains(".globl zip_main"));
}

#[test]
fn н03_импульс_move() {
    let т = asm("САПАР >> ЧАТАВ");
    assert!(т.contains("impulse/move"));
}

#[test]
fn н04_фиксация_perfect() {
    let т = asm("ЧАТАВ!");
    assert!(т.contains("PERFECT") || т.contains("invariant.start"));
}

#[test]
fn н05_вызов_функции() {
    let т = asm("САПАР");
    assert!(т.contains("call zip_"));
}

#[test]
fn н06_хор() {
    let т = asm("ЖАВАТ~ || САГАН~");
    assert!(т.contains("choir spawn"));
}

#[test]
fn н07_константа() {
    let т = asm("САПАР(42)");
    assert!(т.contains("mov rax, 42"));
}

#[test]
fn н08_отладка_loc() {
    let т = asm("САПАР");
    assert!(т.contains(".loc") || т.contains(".file"));
}

#[test]
fn н09_запись_файла_s() {
    let функции = сгенерировать_из_исходника("САПАР >> ЧАТАВ!").unwrap();
    let mut ген = ГенераторНативногоКода::новый("demo");
    ген.сгенерировать(&функции).unwrap();
    let путь = std::env::temp_dir().join("zip_demo.s");
    ген.записать_ассемблер(путь.to_str().unwrap()).unwrap();
    let содержимое = fs::read_to_string(&путь).unwrap();
    assert!(содержимое.contains("impulse/move") || содержимое.contains("call"));
    let _ = fs::remove_file(путь);
}

#[test]
fn н10_объектный_stub() {
    let функции = сгенерировать_из_исходника("САПАР").unwrap();
    let mut ген = ГенераторНативногоКода::новый("obj");
    ген.сгенерировать(&функции).unwrap();
    let путь = std::env::temp_dir().join("zip_demo.o.s");
    ген.записать_объектный_файл(путь.to_str().unwrap()).unwrap();
    assert!(путь.exists());
    let _ = fs::remove_file(путь);
}

#[test]
fn н11_опции_без_native() {
    let mut опции = ОпцииНативнойСборки::default();
    опции.target_cpu_native = false;
    опции.имя_модуля = "x".into();
    let функции = сгенерировать_из_исходника("САПАР").unwrap();
    let mut ген = ГенераторНативногоКода::с_опциями(опции);
    ген.сгенерировать(&функции).unwrap();
    assert!(ген.текст_ассемблера().contains("generic"));
}

#[test]
fn н12_цепочка_сапар_чатав() {
    let т = asm("САПАР >> ЧАТАВ!");
    assert!(т.contains("call"));
    assert!(т.contains("ret"));
    // сохраняем для отчёта
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/сапар_чатав.s");
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&out, &т);
}
