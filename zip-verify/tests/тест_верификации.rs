//! Тесты статической верификации ZIPB.

use zip_vm::байткод_v3::ИнстрV3;
use zip_verify::perfect::проверить_инструкции as проверить_perfect;
use zip_verify::владение::проверить_инструкции as проверить_владение;
use zip_verify::типы::проверить_инструкции as проверить_типы;

#[test]
fn тест_типы_корректны() {
    let инструкции = vec![
        ИнстрV3::PushI64(1),
        ИнстрV3::PushI64(2),
        ИнстрV3::Add,
        ИнстрV3::Ret,
    ];
    assert!(проверить_типы(&инструкции).is_ok());
}

#[test]
fn тест_типы_несовместимы() {
    let инструкции = vec![
        ИнстрV3::PushI64(1),
        ИнстрV3::PushStr(0),
        ИнстрV3::Add,
        ИнстрV3::Ret,
    ];
    assert!(проверить_типы(&инструкции).is_err());
}

#[test]
fn тест_регистры_load_store() {
    let инструкции = vec![
        ИнстрV3::PushI64(42),
        ИнстрV3::Store(0),
        ИнстрV3::Load(0),
        ИнстрV3::Ret,
    ];
    assert!(проверить_типы(&инструкции).is_ok());
}

#[test]
fn тест_владение_после_pulse() {
    let ok = vec![
        ИнстрV3::PushI64(1),
        ИнстрV3::Store(0),
        ИнстрV3::Pulse { из: 0, в: 1 },
        ИнстрV3::Load(1),
        ИнстрV3::Ret,
    ];
    assert!(проверить_владение(&ok).is_ok());

    let bad = vec![
        ИнстрV3::PushI64(1),
        ИнстрV3::Store(0),
        ИнстрV3::Pulse { из: 0, в: 1 },
        ИнстрV3::Load(0), // use-after-move
        ИнстрV3::Ret,
    ];
    assert!(проверить_владение(&bad).is_err());
}

#[test]
fn тест_perfect_не_модифицируется() {
    let ok = vec![
        ИнстрV3::PushI64(7),
        ИнстрV3::Store(0),
        ИнстрV3::PerfectLock(0),
        ИнстрV3::Load(0),
        ИнстрV3::Ret,
    ];
    assert!(проверить_perfect(&ok).is_ok());

    let bad = vec![
        ИнстрV3::PushI64(7),
        ИнстрV3::Store(0),
        ИнстрV3::PerfectLock(0),
        ИнстрV3::PushI64(9),
        ИнстрV3::Store(0), // запрещено
        ИнстрV3::Ret,
    ];
    assert!(проверить_perfect(&bad).is_err());
}

#[test]
fn тест_пустой_стек() {
    let инструкции = vec![ИнстрV3::Add, ИнстрV3::Ret];
    assert!(проверить_типы(&инструкции).is_err());
}
