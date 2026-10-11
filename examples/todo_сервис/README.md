# TODO-Сервис на ZIP

REST API для управления задачами. Архитектура модулей — на ZIP; HTTP/JSON/хранилище — STD runtime `todo_api` (clang -O3), в духе остальных production-модулей экосистемы ZIP.

## Возможности

- CRUD задач
- Аутентификация Bearer-токеном
- Хранение в `данные/задачи.json`
- Логирование запросов в stderr
- CORS для простых клиентов

## Сборка и запуск

```powershell
cd examples/todo_сервис
# из корня репозитория:
../zipc/target/release/zipc.exe собрать src/главная.zip -o сервер
./сервер
# или:
../zipc/target/release/zipc.exe запустить src/главная.zip
```

Сервис: http://localhost:8080

Переменная `ZIP_TODO_DB` — путь к JSON-БД (по умолчанию `данные/задачи.json`).

## API

### Вход

```bash
curl -X POST http://localhost:8080/api/вход \
  -H "Content-Type: application/json" \
  -d "{\"имя\": \"admin\", \"пароль\": \"admin\"}"
```

### Создание задачи

```bash
curl -X POST http://localhost:8080/api/задачи \
  -H "Authorization: Bearer <токен>" \
  -H "Content-Type: application/json" \
  -d "{\"заголовок\": \"Купить молоко\", \"описание\": \"2 литра\"}"
```

### Список / получение / обновление / удаление

```bash
curl -H "Authorization: Bearer <токен>" http://localhost:8080/api/задачи
curl -H "Authorization: Bearer <токен>" http://localhost:8080/api/задачи/1
curl -X PUT -H "Authorization: Bearer <токен>" -H "Content-Type: application/json" \
  -d "{\"выполнена\": true}" http://localhost:8080/api/задачи/1
curl -X DELETE -H "Authorization: Bearer <токен>" http://localhost:8080/api/задачи/1
```

### Health

```bash
curl http://localhost:8080/health
```

## Тесты

```powershell
pwsh ./тестировать.ps1
# только хранилище:
../../zipc/target/release/zipc.exe запустить tests/тест_хранилища.zip
```

## Структура

```
todo_сервис/
├── зипм.манифест
├── src/          — ZIP-модули (контракт + точка входа)
├── данные/       — JSON DB
├── tests/
└── README.md
```

Документация: [docs/ПРИМЕР-ВЕБ-СЕРВИС.md](../../docs/ПРИМЕР-ВЕБ-СЕРВИС.md)
