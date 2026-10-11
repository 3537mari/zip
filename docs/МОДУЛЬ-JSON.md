# Модуль JSON в ZIP STD

## Возможности

- Разбор JSON из строки (`zip_json_разобрать`)
- Запись JSON в строку (`zip_json_записать`)
- Доступ по пути: `user.name`, `items[0]`, `адрес.улица`
- Установка значения по пути (`zip_json_установить`)
- Типы: объект, массив, строка, число, флаг (`true`/`false`), `null`

## Использование

```zip
#runtime("json_parser")
// или
#зависимость("json = 1.0")
```

Сборка демо (компилирует C-runtime с `ZIP_JSON_MAIN`):

```bash
zipc собрать examples/json_демо.zip -o json
./json   # .exe на Windows
```

Ожидаемый вывод:

```text
Имя:
Иван
Возраст:
25
Улица:
Ленина
```

## API (ZIP-обёртки в `zip-std/json.zip`)

| Функция (кириллица) | Описание |
|---------------------|----------|
| `разобрать_йсон(строка, длина)` | → значение JSON |
| `записать_йсон(значение)` | → строка JSON |
| `получить(значение, путь)` | значение по пути |
| `установить(значение, путь, новое)` | запись по пути |
| `это_объект` / `это_массив` / … / `это_пусто` | проверки типа |
| `как_строка` / `как_число` / `как_флаг` | извлечение |

> Идентификаторы ZIP — только кириллица; латиница `json` в именах запрещена → `йсон`.

## C API (`zip-std/runtime/json_parser.c`)

```c
ZipJsonЗначение* zip_json_разобрать(const char* s, size_t n);
char*            zip_json_записать(ZipJsonЗначение* v);
ZipJsonЗначение* zip_json_получить(ZipJsonЗначение* v, const char* path);
int              zip_json_установить(ZipJsonЗначение* root, const char* path, ZipJsonЗначение* neu);
void             zip_json_освободить(ZipJsonЗначение* v);
```

Константы типов: `JSON_ОБЪЕКТ=1` … `JSON_NULL=6`.

## Файлы

- `zip-std/json.zip` — модуль STD
- `zip-std/runtime/json_parser.c` — runtime
- `examples/json_демо.zip` — демо
- `tests/тест_json.rs` — тесты

## См. также

- [STD.md](STD.md)
- [COOKBOOK.md](COOKBOOK.md)
