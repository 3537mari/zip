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

## Статус v4.0.0

- Управление конвейером — **на ZIP**
- Движок lex/parse/codegen — **C-bootstrap** (тот же, что stage-0), вызываемый через `zip_self_*`
- Fixed-point на ZIPB `главное.zip` достигается, потому что stage-1 использует тот же детерминированный компилятор
- Полный порт алгоритмов в чистый ZIP (без builtins) — план v5

См. [docs/ОТЧЁТ-v4.md](../docs/ОТЧЁТ-v4.md), [docs/САМОКОМПИЛЯЦИЯ.md](../docs/САМОКОМПИЛЯЦИЯ.md).
