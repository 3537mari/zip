//! Бенчмарки этапов компиляции ZIP (лексика / парсер / IR / ZIPB).
//!
//! Запуск: `cargo bench --manifest-path zipc/Cargo.toml --bench компиляция`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::time::Duration;
use zipc::байткод_эмиттер_v3;
use zipc::{Лексер, разобрать_исходник, сгенерировать_из_исходника};

fn сгенерировать_программу(строк: usize) -> String {
    let mut программа = String::with_capacity(строк * 40);
    программа.push_str("СПР = { А-А: САПАР, И-А: СИПРА }\n\n");

    for и in 0..строк {
        программа.push_str(&format!(
            "функция ф{}() {{ возврат({} + {}) }}\n",
            и,
            и,
            и * 2
        ));
    }

    программа.push_str("\nфункция главная() {\n");
    for и in 0..строк {
        программа.push_str(&format!("    ф{}()\n", и));
    }
    программа.push_str("}\n");
    программа
}

fn бенчмарк_лексики(c: &mut Criterion) {
    let mut группа = c.benchmark_group("лексика");
    группа.sample_size(20);
    группа.measurement_time(Duration::from_secs(8));

    for размер in [100usize, 1000, 5000] {
        let программа = сгенерировать_программу(размер);
        группа.bench_with_input(BenchmarkId::from_parameter(размер), &программа, |б, код| {
            б.iter(|| {
                let mut лексер = Лексер::новый(код);
                let т = лексер.разобрать().expect("лексика");
                black_box(т.len())
            });
        });
    }
    группа.finish();
}

fn бенчмарк_парсера(c: &mut Criterion) {
    let mut группа = c.benchmark_group("парсер");
    группа.sample_size(10);
    группа.measurement_time(Duration::from_secs(12));

    for размер in [100usize, 1000, 5000] {
        let программа = сгенерировать_программу(размер);
        группа.bench_with_input(BenchmarkId::from_parameter(размер), &программа, |б, код| {
            б.iter(|| {
                let д = разобрать_исходник(код).expect("парсер");
                black_box(д)
            });
        });
    }
    группа.finish();
}

fn бенчмарк_ir(c: &mut Criterion) {
    let mut группа = c.benchmark_group("ir");
    группа.sample_size(10);
    группа.measurement_time(Duration::from_secs(12));

    for размер in [100usize, 1000, 5000] {
        let программа = сгенерировать_программу(размер);
        группа.bench_with_input(BenchmarkId::from_parameter(размер), &программа, |б, код| {
            б.iter(|| {
                let ф = сгенерировать_из_исходника(код).expect("IR");
                black_box(ф.len())
            });
        });
    }
    группа.finish();
}

fn бенчмарк_байткода(c: &mut Criterion) {
    let mut группа = c.benchmark_group("байткод_v3");
    группа.sample_size(10);
    группа.measurement_time(Duration::from_secs(12));

    for размер in [100usize, 1000, 5000] {
        let программа = сгенерировать_программу(размер);
        let функции = сгенерировать_из_исходника(&программа).expect("IR");
        группа.bench_with_input(BenchmarkId::from_parameter(размер), &функции, |б, ф| {
            б.iter(|| {
                let байты = байткод_эмиттер_v3::сгенерировать_v3(ф).expect("zipb");
                black_box(байты.len())
            });
        });
    }
    группа.finish();
}

criterion_group!(
    бенчмарки_компиляции,
    бенчмарк_лексики,
    бенчмарк_парсера,
    бенчмарк_ir,
    бенчмарк_байткода,
);
criterion_main!(бенчмарки_компиляции);
