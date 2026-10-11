//! Бенчмарки компилятора ZIP (парсинг / SIMD-анализ).
//!
//! Запуск: `cargo bench --manifest-path zipc/Cargo.toml --bench производительность`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use std::time::Duration;
use zipc::пул_памяти::ПулПамяти;
use zipc::simd::АнализаторSIMD;
use zipc::{разобрать_исходник, разобрать_исходник_с_пулом, сгенерировать_из_исходника};

fn сгенерировать_программу(размер: usize) -> String {
    let mut out = String::with_capacity(размер * 48);
    out.push_str("функция главная() {\n");
    out.push_str("    с = 0\n");
    for i in 0..размер {
        out.push_str(&format!("    с = с + {}\n", i % 97));
    }
    out.push_str("    возврат(с)\n}\n");
    out
}

fn бенчмарк_компиляции(c: &mut Criterion) {
    let mut группа = c.benchmark_group("компиляция");
    группа.sample_size(10);
    группа.measurement_time(Duration::from_secs(10));

    for размер in [100usize, 1000, 5000].iter() {
        let исходник = сгенерировать_программу(*размер);
        группа.bench_with_input(BenchmarkId::new("парсинг", размер), размер, |б, _| {
            б.iter(|| {
                let _ = разобрать_исходник(&исходник);
            });
        });
        группа.bench_with_input(
            BenchmarkId::new("парсинг_пул", размер),
            размер,
            |б, _| {
                б.iter(|| {
                    let пул = Arc::new(ПулПамяти::новый());
                    let _ = разобрать_исходник_с_пулом(&исходник, пул);
                });
            },
        );
    }

    группа.finish();
}

fn бенчмарк_simd(c: &mut Criterion) {
    let исходник = r#"
        функция сумма() {
            с = 0
            для и в 0..100000 {
                с = с + и * 2
            }
            возврат(с)
        }
    "#;
    let функции = сгенерировать_из_исходника(исходник).expect("IR");
    let анализатор = АнализаторSIMD::новый();

    c.bench_function("simd_анализ_циклов", |б| {
        б.iter(|| {
            let _ = анализатор.найти_векторизуемые(&функции[0]);
        });
    });
}

criterion_group!(бенчмарки, бенчмарк_компиляции, бенчмарк_simd);
criterion_main!(бенчмарки);
