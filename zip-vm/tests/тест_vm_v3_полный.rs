//! Покрытие основных опкодов VM v3.

use zip_vm::байткод_v3::{ИнстрV3, МодульV3, ФункцияV3};
use zip_vm::исполнитель_v3::{ИсполнительV3, ЗначениеV3};

fn функция(имя_индекс: u32, арность: u16, инструкции: Vec<ИнстрV3>) -> ФункцияV3 {
    let mut код = Vec::new();
    for и in &инструкции {
        и.закодировать(&mut код);
    }
    ФункцияV3 {
        имя_индекс,
        арность,
        локалы: 16,
        флаги: 0,
        код,
    }
}

fn модуль(инструкции: Vec<ИнстрV3>) -> МодульV3 {
    МодульV3 {
        флаги: 0,
        строки: vec!["главная".into(), "тест".into()],
        функции: vec![функция(0, 0, инструкции)],
    }
}

fn запуск(инструкции: Vec<ИнстрV3>) -> ЗначениеV3 {
    let mut и = инструкции;
    if !и.iter().any(|x| matches!(x, ИнстрV3::Ret)) {
        и.push(ИнстрV3::Ret);
    }
    let mut vm = ИсполнительV3::новый(модуль(и)).unwrap();
    vm.запустить().unwrap()
}

#[test]
fn тест_push_i64() {
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(42)]),
        ЗначениеV3::Целое(42)
    );
}

#[test]
fn тест_push_f64() {
    let р = запуск(vec![ИнстрV3::PushF64(3.14)]);
    match р {
        ЗначениеV3::Вещественное(f) => assert!((f - 3.14).abs() < 1e-9),
        другой => panic!("{другой:?}"),
    }
}

#[test]
fn тест_push_str() {
    let модуль = МодульV3 {
        флаги: 0,
        строки: vec!["главная".into(), "привет".into()],
        функции: vec![функция(
            0,
            0,
            vec![ИнстрV3::PushStr(1), ИнстрV3::Ret],
        )],
    };
    let mut vm = ИсполнительV3::новый(модуль).unwrap();
    assert_eq!(vm.запустить().unwrap(), ЗначениеV3::Строка("привет".into()));
}

#[test]
fn тест_push_bool_null() {
    assert_eq!(запуск(vec![ИнстрV3::PushTrue]), ЗначениеV3::Флаг(true));
    assert_eq!(запуск(vec![ИнстрV3::PushFalse]), ЗначениеV3::Флаг(false));
    assert_eq!(запуск(vec![ИнстрV3::PushNull]), ЗначениеV3::Ничто);
}

#[test]
fn тест_add_sub_mul_div_mod() {
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(2),
            ИнстрV3::PushI64(3),
            ИнстрV3::Add
        ]),
        ЗначениеV3::Целое(5)
    );
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(10),
            ИнстрV3::PushI64(3),
            ИнстрV3::Sub
        ]),
        ЗначениеV3::Целое(7)
    );
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(3),
            ИнстрV3::PushI64(4),
            ИнстрV3::Mul
        ]),
        ЗначениеV3::Целое(12)
    );
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(10),
            ИнстрV3::PushI64(2),
            ИнстрV3::Div
        ]),
        ЗначениеV3::Целое(5)
    );
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(10),
            ИнстрV3::PushI64(3),
            ИнстрV3::Mod
        ]),
        ЗначениеV3::Целое(1)
    );
}

#[test]
fn тест_сравнения() {
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(5), ИнстрV3::PushI64(5), ИнстрV3::Eq]),
        ЗначениеV3::Флаг(true)
    );
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(5), ИнстрV3::PushI64(3), ИнстрV3::Ne]),
        ЗначениеV3::Флаг(true)
    );
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(3), ИнстрV3::PushI64(5), ИнстрV3::Lt]),
        ЗначениеV3::Флаг(true)
    );
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(3), ИнстрV3::PushI64(5), ИнстрV3::Le]),
        ЗначениеV3::Флаг(true)
    );
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(5), ИнстрV3::PushI64(3), ИнстрV3::Gt]),
        ЗначениеV3::Флаг(true)
    );
    assert_eq!(
        запуск(vec![ИнстрV3::PushI64(5), ИнстрV3::PushI64(5), ИнстрV3::Ge]),
        ЗначениеV3::Флаг(true)
    );
}

#[test]
fn тест_dup_swap_pop() {
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(1),
            ИнстрV3::Dup,
            ИнстрV3::Add
        ]),
        ЗначениеV3::Целое(2)
    );
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(1),
            ИнстрV3::PushI64(2),
            ИнстрV3::Swap,
            ИнстрV3::Sub
        ]),
        ЗначениеV3::Целое(1)
    ); // после swap: 2,1 → 2-1=1
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(9),
            ИнстрV3::PushI64(1),
            ИнстрV3::Pop
        ]),
        ЗначениеV3::Целое(9)
    );
}

#[test]
fn тест_load_store() {
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(7),
            ИнстрV3::Store(0),
            ИнстрV3::Load(0)
        ]),
        ЗначениеV3::Целое(7)
    );
}

#[test]
fn тест_jump_if_not() {
    // 0: Push 0, 1: JumpIfNot(4), 2: Push 1, 3: Jump(5), 4: Push 99, 5: Ret
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(0),
            ИнстрV3::JumpIfNot(4),
            ИнстрV3::PushI64(1),
            ИнстрV3::Jump(5),
            ИнстрV3::PushI64(99),
            ИнстрV3::Ret,
        ]),
        ЗначениеV3::Целое(99)
    );
}

#[test]
fn тест_new_something_success_error_nothing() {
    assert!(matches!(
        запуск(vec![ИнстрV3::PushI64(1), ИнстрV3::NewSomething]),
        ЗначениеV3::Нечто(_)
    ));
    assert!(matches!(
        запуск(vec![ИнстрV3::PushI64(1), ИнстрV3::NewSuccess]),
        ЗначениеV3::Успех(_)
    ));
    assert!(matches!(
        запуск(vec![ИнстрV3::PushStr(1), ИнстрV3::NewError]),
        ЗначениеV3::Ошибка(_)
    ));
    assert_eq!(запуск(vec![ИнстрV3::NewNothing]), ЗначениеV3::Ничто);
}

#[test]
fn тест_perfect_lock() {
    assert!(matches!(
        запуск(vec![
            ИнстрV3::PushI64(5),
            ИнстрV3::Store(0),
            ИнстрV3::PerfectLock(0),
            ИнстрV3::Load(0),
        ]),
        ЗначениеV3::Совершенное(_)
    ));
}

#[test]
fn тест_pulse() {
    assert_eq!(
        запуск(vec![
            ИнстрV3::PushI64(11),
            ИнстрV3::Store(0),
            ИнстрV3::Pulse { из: 0, в: 1 },
            ИнстрV3::Load(1),
        ]),
        ЗначениеV3::Целое(11)
    );
}

#[test]
fn тест_new_set_iter() {
    assert!(matches!(запуск(vec![ИнстрV3::NewSet]), ЗначениеV3::Множество(_)));
    assert!(matches!(
        запуск(vec![ИнстрV3::NewSet, ИнстрV3::NewIter]),
        ЗначениеV3::Итератор(_, _)
    ));
}

#[test]
fn тест_nop() {
    assert_eq!(
        запуск(vec![ИнстрV3::Nop, ИнстрV3::PushI64(1)]),
        ЗначениеV3::Целое(1)
    );
}

#[test]
fn тест_call_ret() {
    let модуль = МодульV3 {
        флаги: 0,
        строки: vec!["главная".into(), "плюс1".into()],
        функции: vec![
            функция(
                0,
                0,
                vec![
                    ИнстрV3::PushI64(41),
                    ИнстрV3::Call(1),
                    ИнстрV3::Ret,
                ],
            ),
            функция(
                1,
                1,
                vec![
                    ИнстрV3::Load(0),
                    ИнстрV3::PushI64(1),
                    ИнстрV3::Add,
                    ИнстрV3::Ret,
                ],
            ),
        ],
    };
    let mut vm = ИсполнительV3::новый(модуль).unwrap();
    assert_eq!(vm.запустить().unwrap(), ЗначениеV3::Целое(42));
}

#[test]
fn тест_print_i64_не_падает() {
    let _ = запуск(vec![ИнстрV3::PushI64(1), ИнстрV3::PrintI64, ИнстрV3::PushI64(0)]);
}

#[test]
fn тест_check_tag() {
    let р = запуск(vec![
        ИнстрV3::PushI64(1),
        ИнстрV3::NewSomething,
        ИнстрV3::CheckTag,
    ]);
    // CheckTag оставляет значение и кладёт имя типа строкой
    assert!(matches!(р, ЗначениеV3::Строка(_)));
}
