# zip-self-zip — Bootstrap-компилятор на ZIP

Конвейер самокомпиляции ZIP → ZIPB:

| Модуль | Роль |
|--------|------|
| `лексика.zip` | API лексера (`лексер_разобрать`) |
| `парсер.zip` | API парсера |
| `генератор.zip` | API генератора ZIPB |
| `главное.zip` | CLI-точка входа |

## Запуск

```bash
# 1. C-bootstrap собирает selfc
./bootstrap/zipc zip-self-zip/главное.zip /tmp/selfc_v1.zipb

# 2. Selfc компилирует себя (через VM + тот же движок)
./zip-vm/target/release/zip-vm /tmp/selfc_v1.zipb \
    zip-self-zip/главное.zip /tmp/selfc_v2.zipb

# 3. Fixed-point: хеши v1 == v2
```

Windows: `pwsh zip-self-zip/тесты/тест_fixed_point.ps1`

## Статус

- Управление конвейером — **на ZIP**
- **Лексер:** полный алгоритм без C-bootstrap — [`лексика_полная.zip`](лексика_полная.zip) + [`zip_lex_полный`](../zip-vm/src/zip_lex_полный.rs)
- Парсер / codegen на stage-1 пока через C-bootstrap (`zip_self_compile`)
- Fixed-point `главное.zip` сохранён
- Полный порт parse/codegen — [ROADMAP_V5.md](../ROADMAP_V5.md)

См. [docs/ОТЧЁТ-v4.md](../docs/ОТЧЁТ-v4.md), [docs/САМОКОМПИЛЯЦИЯ.md](../docs/САМОКОМПИЛЯЦИЯ.md).
