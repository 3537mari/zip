# ZIP — язык программирования на кириллице

[![Версия](https://img.shields.io/badge/версия-3.0.0-blue)](CHANGELOG.md)
[![Тесты](https://img.shields.io/badge/тесты-230%2B%20✅-green)](tests/)
[![Лицензия](https://img.shields.io/badge/лицензия-MIT-green)](LICENSE)
[![Самокомпиляция](https://img.shields.io/badge/самокомпиляция-✅-green)](docs/САМОКОМПИЛЯЦИЯ.md)

**ZIP** (Zero-Immunity-Pulse) — императивный потоковый язык
программирования на кириллице. Программа — семантический поток
действий (VSO), а не набор объектов.

## Ключевые возможности

- **Скорость** — LLVM-бэкенд, ≈19× быстрее ASM
- **Защита** — постквантовая криптография, анализ владения
- **ИИ** — встроенные примитивы для нейросетей
- **Потоковость** — VSO-синтаксис, `>>` и `||`
- **Биология** — 64 корня, комплементарность, эпигенетика
- **Самокомпиляция** — компилятор на самом себе
- **Кириллица** — только русские буквы в коде

## Быстрый старт

### Установка

#### Вариант 1: C-bootstrap (минимальный)

**Требования:** только C-компилятор (gcc/clang).

```bash
git clone https://github.com/3537mari/zip.git
cd zip/bootstrap
make
./zipc ../zip-self/компилятор.zip /tmp/selfc.zipb
```

Готово! Компилятор `zipc` собран без Rust и LLVM.

#### Вариант 2: Готовые пакеты

Скачайте с [GitHub Releases](https://github.com/3537mari/zip/releases):
- Windows: `ZIP-3.0.0-x64.msi`
- Debian/Ubuntu: `zip-lang_3.0.0_amd64.deb`
- Red Hat/Fedora: `zip-lang-3.0.0-1.x86_64.rpm`
- macOS: `ZIP-3.0.0-macos.dmg`
- Universal: `ZIP-3.0.0-offline-installer.sh`

Скрипты сборки: [`installers/`](installers/).

#### Вариант 3: Полная сборка (для разработки)

**Требования:** Rust 1.75+.

```bash
cargo build --release --manifest-path zipc/Cargo.toml
cargo build --release --manifest-path zip-vm/Cargo.toml
```

### Проверка

```bash
zipc --version
# ZIP compiler v3.0.0 / ZIP bootstrap compiler v3.1.0

echo 'ПечатьСтроки("Привет, мир!")' > тест.zip
zipc собрать тест.zip -o тест
zip-vm тест.zipb
# Привет, мир!
```

### Способы компиляции

| Режим | Команда | Требования | Скорость |
|-------|---------|------------|----------|
| **VM v3** (по умолчанию) | `zipc собрать тест.zip -o тест` | Только C / Rust | ×1 |
| **LLVM** (опционально) | `zipc собрать тест.zip --llvm` | + LLVM | ×1.25 |

### Первая программа

```bash
cat > привет.zip << 'EOF'
ПечатьСтроки("Привет, мир!")
EOF

# По умолчанию (v3): байт-код ZIPB → zip-vm
./zipc/target/release/zipc собрать привет.zip -o привет
./zip-vm/target/release/zip-vm привет.zipb

# Опционально: LLVM
./zipc/target/release/zipc запустить привет.zip --llvm
```

### Нейросеть XOR

```bash
./zipc/target/release/zipc запустить \
    zip-std/примеры/xor_обучение.zip --llvm
```

## Примеры

### Поток данных

```zip
поток обработка {
    источник = ВАДАТ(камера)
    фильтр размытие = САПАР(фильтр_размытия)
    фильтр края = САПАР(детектор_краёв)
    приёмник = ЧАТАВ(результат)
}
```

### Агент

```zip
агент Сканер_Логов {
    состояние {
        последняя_позиция: Целое = 0
    }

    функция слушать() {
        события = ВАДАТ(лог_файл, последняя_позиция)
    }
}
```

### Аннотации

```zip
@ускорить(уровень = 3)
@кэшировать(размер = 1024)
функция дорогая_функция(х) {
    возврат(х * х)
}
```

## Документация

Полный индекс: [docs/ИНДЕКС.md](docs/ИНДЕКС.md)

### Для новичков

- [Учебник «ZIP за 30 минут»](docs/УЧЕБНИК.md)
- [Cookbook — 50 рецептов](docs/COOKBOOK.md)
- [FAQ](docs/FAQ.md)

### Справочники

- [Справочник по языку](docs/СПРАВОЧНИК.md)
- [Справочник по STD](docs/STD.md)
- [Синтаксис](docs/СИНТАКСИС.md)
- [Алфавит](docs/АЛФАВИТ.md)

### Для опытных

- [Миграция с Python/Rust](docs/МИГРАЦИЯ.md)
- [Производительность](docs/ПРОИЗВОДИТЕЛЬНОСТЬ.md)
- [Экосистема](docs/ЭКОСИСТЕМА.md)
- [Биологические параллели](docs/БИОЛОГИЯ.md)

### Планы

- [Мастер-план](МАСТЕР-ПЛАН.md)
- [Дорожная карта](ROADMAP.md)
- [CHANGELOG](CHANGELOG.md)

## Сообщество

- [Discord](https://discord.gg/zip-lang)
- [Telegram-канал](https://t.me/zip_lang_news)
- [Telegram-чат](https://t.me/zip_lang_chat)
- [GitHub Discussions](https://github.com/3537mari/zip/discussions)
- [GitHub Issues](https://github.com/3537mari/zip/issues)
- Инфраструктура: [community/](community/) · [docs/СООБЩЕСТВО.md](docs/СООБЩЕСТВО.md)

## Вклад

См. [CONTRIBUTING.md](CONTRIBUTING.md). Мы рады любому вкладу!

## Кодекс поведения

См. [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Управление

См. [GOVERNANCE.md](GOVERNANCE.md) · [FAQ.md](FAQ.md).

## Лицензия

MIT — см. [LICENSE](LICENSE).
