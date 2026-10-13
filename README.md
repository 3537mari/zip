# ZIP — язык программирования на кириллице

[![Версия](https://img.shields.io/badge/версия-6.1.0-blue)](CHANGELOG.md)
[![ARM64](https://img.shields.io/badge/ARM64-v7.0.0--alpha-brightgreen)](docs/ARM64.md)
[![RISC-V](https://img.shields.io/badge/RISC--V-v7.0.0--beta-brightgreen)](docs/RISCV.md)
[![WASM](https://img.shields.io/badge/WASM-v7.0.0--rc-brightgreen)](docs/WASM.md)
[![Тесты](https://img.shields.io/badge/тесты-300%2B%20✅-green)](tests/)
[![Fixed-point](https://img.shields.io/badge/fixed--point-v2=v3=v4-brightgreen)](zip-self-zip/)
[![Лексер/парсер/генератор](https://img.shields.io/badge/ядро-ZIP-brightgreen)](zip-self-zip/)
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

## Установка v6.1.0

### Вариант 1: Готовые бинарники (рекомендуется)

Скачайте с [GitHub Releases](https://github.com/3537mari/zip/releases/tag/v6.1.0):

- Windows: `ZIP-6.1.0-x64.msi`
- Linux: `zip-lang_6.1.0_amd64.deb`
- macOS: `ZIP-6.1.0-macos.dmg`

После установки:

```bash
zip-vm selfc.zipb программа.zip программа.zipb
zip-vm программа.zipb
```

### Вариант 2: Из исходников (C нужен один раз для stage-0)

```bash
# 1. C-bootstrap (последний раз с C)
cd bootstrap
make   # или: pwsh -File build.ps1
cd ..

# 2. Собрать VM (Rust — только интерпретатор)
cargo build --release --manifest-path zip-vm/Cargo.toml

# 3. Компиляция ZIP-компилятора на чистом ZIP
./bootstrap/zipc zip-self-zip/главное_чистое.zip /tmp/selfc_v1.zipb

# 4. Первая самокомпиляция через VM
./zip-vm/target/release/zip-vm /tmp/selfc_v1.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_v2.zipb

# 5. Вторая самокомпиляция — fixed-point
./zip-vm/target/release/zip-vm /tmp/selfc_v2.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_v3.zipb

# 6. Проверка fixed-point
sha256sum /tmp/selfc_v2.zipb /tmp/selfc_v3.zipb
# Хеши совпадают: 03ec79b9...C6A98C59

# Дальше — только ZIP!
```

Тест: `pwsh zip-self-zip/тесты/тест_fixed_point_без_c.ps1`  
Паритет: `python tools/_parity_v61.py`

### Вариант 3: Чистая установка (только VM)

Если у вас уже есть `zip-vm` (из v6.0.0+):

```bash
# 1. Скачать selfc (ZIP-компилятор)
wget https://github.com/3537mari/zip/releases/download/v6.1.0/selfc.zipb

# 2. Компилировать любую программу
zip-vm selfc.zipb программа.zip программа.zipb
zip-vm программа.zipb
```

**C-компилятор больше не нужен после первой сборки!**

## Честная независимость (v6.1.0)

ZIP — **самокомпилируемый язык**, где компилятор написан на самом ZIP.

| Компонент | Реализация | Язык |
|-----------|------------|------|
| Лексер | `лексика_чистая.zip` | **ZIP** |
| Парсер | `парсер_чистый.zip` | **ZIP** |
| Генератор ZIPB | `генератор_чистый.zip` | **ZIP** |
| Точка входа | `главное_чистое.zip` | **ZIP** |
| Runtime | `zip-std/syscalls.zip` и др. | **ZIP** |
| STD | `zip-std/*.zip` | **ZIP** |
| VM | `zip-vm/src/*.rs` | Rust (только интерпретатор) |
| C-bootstrap | `bootstrap/*.c` | C (только stage-0) |

**Fixed-point:**
- v2 = v3 = v4 = `03ec79b9...C6A98C59`
- v1 ≠ v2 — ожидаемо (C stage-0 vs ZIP-генератор)

**Это означает:**
- ZIP = Rust = Go = Swift (честно, без оговорок)
- C нужен **один раз** для первой сборки
- Rust-VM — **только интерпретатор** ZIP-байткода

Отчёт: [docs/ОТЧЁТ-v6.md](docs/ОТЧЁТ-v6.md).

## Платформы

| Платформа | VM | Статус |
|-----------|-----|--------|
| x86-64 Linux | Rust | ✅ |
| x86-64 Windows | Rust | ✅ |
| x86-64 macOS | Rust | ✅ |
| **ARM64 Linux** | Rust | ✅ (через QEMU / CI) |
| **ARM64 macOS** | Rust | ✅ (target) |
| **ARM64 Windows** | Rust | ✅ (target) |
| **RISC-V 64** | Rust | ✅ (через QEMU / CI) |
| **WASM (браузер)** | **wasm-bindgen** | ✅ (v7.0.0-rc) |

**ZIP-байткод платформонезависим.** Один и тот же `.zipb` работает на всех платформах.  
Подробнее: [docs/ARM64.md](docs/ARM64.md) · [docs/RISCV.md](docs/RISCV.md).

### Способы компиляции

| Режим | Команда | Требования | Скорость |
|-------|---------|------------|----------|
| **selfc (ZIP)** | `zip-vm selfc.zipb вход.zip выход.zipb` | zip-vm | ×1 |
| **C-bootstrap** | `./bootstrap/zipc вход.zip выход.zipb` | stage-0 | ×1 |
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
