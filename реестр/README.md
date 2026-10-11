# Реестр пакетов ZIP

Локальный каталог + HTTP-сервер.

## Запуск сервера

```bash
cargo build --release --manifest-path zipc/Cargo.toml
./zipc/target/release/zipc реестр/сервер/сервер.zip -o реестр/сервер/сервер

# Windows
set ZIP_REGISTRY_ROOT=%CD%\реестр
реестр\сервер\сервер.exe 8080

# Unix
ZIP_REGISTRY_ROOT="$PWD/реестр" ./реестр/сервер/сервер 8080
```

## API

| Метод | Путь | Описание |
|-------|------|----------|
| GET | `/` или `/health` | healthcheck |
| GET | `/индекс` или `/index` | индекс (JSON) |
| GET | `/пакет/{имя}/{версия}` | манифест пакета |
| GET | `/поиск?q=` или `/search?q=` | поиск |
| POST | `/пакет/{имя}/{версия}` | публикация (тело gzip) |

Переменная `ZIPM_REGISTRY=https://…` включает HTTP publish/install в `зипм`.
