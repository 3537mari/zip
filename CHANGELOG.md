# Changelog

Все значительные изменения ZIP документируются здесь.

Формат: [Keep a Changelog](https://keepachangelog.com/ru/1.0.0/),
версионирование: [Semantic Versioning](https://semver.org/lang/ru/).

## [Unreleased]

## [7.2.0] — 2026-10-13

### iOS (Swift + Objective-C bridge)
- Crate `zip-vm-ios` → `libzip_vm_ios.a` (`aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios`)
- `ZipVM.xcframework` (lipo universal simulator + device)
- C-API: `zip_compile`, `zip_run`, `zip_compile_and_run`, `zip_free`, `zip_version`, `zip_last_success`
- Swift: `Zip.compile` / `run` / `compileAndRun` / `version`
- Демо SwiftUI + XCTest на симуляторе
- CI: [`.github/workflows/ios.yml`](.github/workflows/ios.yml)
- Docs: [docs/IOS.md](docs/IOS.md)

## [7.1.0] — 2026-10-13

### Android (NDK + JNI)
- Crate `zip-vm-android` → `libzip_vm_android.so` (arm64-v8a, x86_64)
- Публичный API `zip_vm::zip_компилятор`
- Kotlin: `Zip.скомпилировать` / `запустить` / `скомпилироватьИЗапустить` / `версия`
- CI: [`.github/workflows/android.yml`](.github/workflows/android.yml)
- Docs: [docs/ANDROID.md](docs/ANDROID.md)

## [7.0.0] — 2026-10-13

### Кросс-компиляция Windows ARM64 — финал v7
- Target: `aarch64-pc-windows-msvc`
- CI: [`.github/workflows/windows-arm.yml`](.github/workflows/windows-arm.yml)
  — нативный `windows-11-arm` + cross-build на `windows-latest`
- Docs: [docs/WINDOWS-ARM.md](docs/WINDOWS-ARM.md)
- **v7.0.0 закрыт:** x86-64 · ARM64 (Linux/macOS/Windows) · RISC-V · WASM

## [7.0.0-rc] — 2026-10-13

### Кросс-компиляция WASM (VM в браузере)
- Rust-VM: `wasm32-unknown-unknown` + `wasm-bindgen` (`--features wasm`)
- Web-backend: захват вывода без `std::process` / файлов
- Плейграунд: компилировать **и запускать** ZIP (`compileAndRun`)
- CI: `.github/workflows/wasm.yml` (headless Chrome)
- Docs: [docs/WASM.md](docs/WASM.md)

## [7.0.0-beta] — 2026-10-13

### Кросс-компиляция RISC-V (VM)

- Target: `riscv64gc-unknown-linux-gnu`
- CI: [`.github/workflows/riscv.yml`](.github/workflows/riscv.yml) — сборка + QEMU
- Документация: [docs/RISCV.md](docs/RISCV.md)

**Доказательство:** `selfc` на RISC-V через QEMU даёт тот же хеш `03ec79b9...c6a98c59`, что x86-64 и ARM64.

## [7.0.0-alpha] — 2026-10-13

### Кросс-компиляция ARM64 (VM)

- Targets: `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `aarch64-pc-windows-msvc`
- CI: [`.github/workflows/arm64.yml`](.github/workflows/arm64.yml) — сборка + QEMU
- `zip-vm --target …` — информационный флаг (ZIPB платформонезависим)
- Документация: [docs/ARM64.md](docs/ARM64.md)

**Доказательство платформонезависимости:** один и тот же `selfc.zipb` на ARM64 через QEMU даёт байткод с хешем `03ec79b9...` (как на x86-64).

## [6.1.0] — 2026-10-12

### Полный lowering лексера/парсера/генератора в ZIP-байткод

**Что это значит:**
- Лексер — алгоритм на чистом ZIP (`zip-self-zip/лексика_чистая.zip`)
- Парсер — алгоритм на чистом ZIP (`zip-self-zip/парсер_чистый.zip`)
- Генератор ZIPB — алгоритм на чистом ZIP (`zip-self-zip/генератор_чистый.zip`)
- ZIP-модули больше **НЕ вызывают Rust-функции компилятора** (`zip_self_lex` / `zip_self_parse` / `zip_self_gen` в `zip_self_host.rs` — только маркеры)
- Rust-VM — **только интерпретатор**, а не компилятор

**Fixed-point без Rust-функций компилятора:**
- v2 = v3 = v4 = `03ec79b9...C6A98C59`
- v1 ≠ v2 — ожидаемо (C-bootstrap stage-0 vs ZIP-генератор)
- Fixed-point достигается **через ZIP-алгоритм**, а не через Rust

**Ключевые фиксы:**
- Постфикс `[` для корней/шаблонов — `поз[0]` больше не ломает `виды[поз[0]]`
- `CallNative` берёт арность сверху стека (байтбуферы)
- C-bootstrap: `POP` после statement-CALL

**Следствие:**
- «ZIP = Rust = Go = Swift» — теперь **честно**, без звёздочки
- ARM64/WASM — задача **бэкенда** (Rust-VM), а не порта всего компилятора

**Тесты:** `zip-self-zip/тесты/тест_fixed_point_без_c.ps1`, `tools/_parity_v61.py`

**Ссылки:**
- Коммит: `95d6395`
- Релиз: https://github.com/3537mari/zip/releases/tag/v6.1.0

## [6.0.0] — 2026-10-12

### Полная независимость от C (stage-1+)

**Structs / массивы / указатели** в C-bootstrap + VM (опкоды 0x93–0xA3).

**Компилятор на ZIP:**
- `zip-self-zip/лексика_чистая.zip`, `парсер_чистый.zip`, `генератор_чистый.zip`, `главное_чистое.zip`
- `zip_self_compile` → VM `zip_gen_полный` (без вызова C; `ZIP_USE_C_BOOTSTRAP=1` — legacy)

**Fixed-point:** v2 == v3 при самокомпиляции через VM.

См. [ОТЧЁТ-v6.md](docs/ОТЧЁТ-v6.md), [ROADMAP_V7.md](ROADMAP_V7.md).

## [5.0.0] — 2026-10-12

### Конвейер компилятора в VM (без вызова C для lex/parse/gen)

- Лексер: `zip_lex_полный.rs` + `zip_self_lex`
- Парсер: `zip_parse_полный.rs`, `zip_ast.rs` + `zip_self_parse`
- Генератор: `zip_gen_полный.rs` + `zip-vm --gen` / `zip_self_gen`
- Тесты цепочки: `тест_лексера_zip`, `тест_парсера_zip`, `тест_генератора_zip`, `тест_полной_цепочки_zip`
- Docs: [ОТЧЁТ-v5.md](docs/ОТЧЁТ-v5.md), [ГЕНЕРАТОР-В-VM.md](docs/ГЕНЕРАТОР-В-VM.md)

**Граница:** stage-0 установки по-прежнему через C-bootstrap; полный lowering в ZIP — v6.

## [4.0.0] — 2026-10-12

### Runtime / STD / Bootstrap-конвейер на ZIP

**Runtime:**
- Syscalls, bump allocator, вывод, файлы — ZIP API ([RUNTIME-НА-ZIP.md](docs/RUNTIME-НА-ZIP.md))

**STD:**
- `*_zip.zip`: ядро, тензоры, нейросети, крипто, json/csv/regex, сеть
- `zip_std_v4` + inline syscalls (`syscalls_inline`)

**Bootstrap (`zip-self-zip/`):**
- Лексер / парсер / генератор / `главное.zip` — конвейер на ZIP
- Fixed-point: C-bootstrap → selfc → selfc (одинаковый ZIPB)
- Движок stage-0/1: тот же `bootstrap/zipc` через `zip_self_compile` (полный порт алгоритмов — v5)

**VM:**
- Аргументы программы (`получить_аргумент`)
- Каркас `zip-vm-zip/`

**Документация:** [ОТЧЁТ-v4.md](docs/ОТЧЁТ-v4.md), [STD-НА-ZIP.md](docs/STD-НА-ZIP.md), [ROADMAP_V5.md](ROADMAP_V5.md)

### Установка (fixed-point)

```bash
./bootstrap/zipc zip-self-zip/главное.zip /tmp/selfc_v1.zipb
./zip-vm/target/release/zip-vm /tmp/selfc_v1.zipb \
  zip-self-zip/главное.zip /tmp/selfc_v2.zipb
```

## [3.1.0] — 2026-10-11

### Расширенный C-bootstrap

- Полный лексер: ключевые слова, операторы (`>>`, `>>>`, `||`, `!`, `..`, `@`, `#`)
- Полный парсер: функции, если/пока/для, приоритеты, вызовы, присваивания (в т.ч. TK_TEMPLATE)
- Генератор ZIPB v3: арифметика, сравнения, JUMP/CALL/RET, циклы
- Fixed-point: детерминированная перекомпиляция (`bootstrap/тесты/тест_fixed_point.*`)
- Сравнение с Rust-bootstrap (`bootstrap/тесты/сравнить_с_rust.*`)
- План v4: [ROADMAP_V4.md](ROADMAP_V4.md)

## [3.0.0] — 2026-10-11

### Bootstrap на C

- Минимальный компилятор на C: лексер → парсер → AST → ZIPB v3 (`bootstrap/`)
- Сборка: `make` / `pwsh bootstrap/build.ps1` (clang/gcc)
- Без Rust и LLVM для bootstrap-пути

### LLVM опционален

- По умолчанию `zipc` пишет `.zipb` (VM v3)
- Нативный LLVM-путь: `--llvm` / `--simd`

### Installers

- Windows WiX (`.msi`), Debian (`.deb`), RPM, macOS (`.dmg`), offline (`.sh` / `.ps1`)
- CI: `build-installers.yml`

## [2.2.0] — 2026-10-11

### Сообщество и обучение

#### Обучение

- Интерактивный курс «ZIP за 30 минут» (8 уроков) на сайте
- WASM-плейграунд, прогресс, проверка задач, мобильная вёрстка, тёмная тема
- Сценарии 5 видео-туториалов + план записи
- Социальные шаблоны и SVG

#### Публикации

- Тексты для HN, Reddit, Habr, Twitter, конференций (`publishing/`)
- Сводка: [docs/ПУБЛИКАЦИИ.md](docs/ПУБЛИКАЦИИ.md)

#### Сообщество

- Инфраструктура Discord / Telegram (`community/`)
- Шаблоны GitHub Discussions
- CONTRIBUTING.md, CODE_OF_CONDUCT.md, GOVERNANCE.md, FAQ.md
- [docs/СООБЩЕСТВО.md](docs/СООБЩЕСТВО.md)

#### Инфраструктура

- Заглушки JIT для wasm32 (деплой GitHub Pages)

## [2.1.0] — 2026-10-11

### Production-ready

#### Производительность

- Оптимизация zipc: look-ahead без clone, без двойной лексики, HashMap в ZIPB-эмиттере
- Пайплайн компиляции ~1.9–3.0× быстрее ([ОПТИМИЗАЦИЯ-КОМПИЛЯТОРА](docs/ОПТИМИЗАЦИЯ-КОМПИЛЯТОРА.md))

#### Тесты

- Покрытие ядра компилятора **90.02%** (llvm-cov)
- Расширенные тесты лексики / парсера / IR / эмиттеров / VM v3

#### Бенчмарки

- Сравнение ZIP vs Python vs Rust vs Go ([БЕНЧМАРКИ](docs/БЕНЧМАРКИ.md))
- ZIP ≈ **40×** быстрее CPython в среднем по задачам

#### Реальный проект

- TODO REST API на ZIP (`examples/todo_сервис/`, `#runtime("todo_api")`)
- CRUD, Bearer-auth, JSON-файл БД, логирование
- Документация: [ПРИМЕР-ВЕБ-СЕРВИС](docs/ПРИМЕР-ВЕБ-СЕРВИС.md)

## [2.0.0] — 2026-10-11

### Полная независимость

#### Добавлено

- Байт-код ZIP v3 (`.zipb`) и VM v3 без LLVM
- Компилятор ZIP → ZIPB на zip-self (`компилятор.zip`)
- Формальная семантика, теоремы, инструмент `zip-verify`
- Документация: [VM-V3](docs/VM-V3.md), [КОМПИЛЯТОР-НА-ZIP](docs/КОМПИЛЯТОР-НА-ZIP.md),
  [ФОРМАЛЬНАЯ-СЕМАНТИКА](docs/ФОРМАЛЬНАЯ-СЕМАНТИКА.md), [ВЕРИФИКАЦИЯ](docs/ВЕРИФИКАЦИЯ.md)

## [1.1.0] — 2026-10-11

### Документация и производительность

#### Добавлено

- Идея 3: нейрогенез — композитные функции из паттернов IR
- Идея 19: ИИ-советник (`--ии-советник`)
- R7: shebang, `#зависимость`, кэш скриптов
- HTTP-реестр пакетов, сайт, WASM-плейграунд
- SIMD-анализ циклов, `--simd`, параллельный JIT, memory pooling, prefetch
- Документация: учебник, справочник, STD, Cookbook (50), FAQ, миграция, индекс

#### Планы

См. [ROADMAP.md](ROADMAP.md) — v1.1.0 завершён; дальше v1.2.0 (STD-модули).

## [1.0.0] — 2026-10-11

### Первый стабильный релиз ZIP

#### Компилятор

- Лексер с полной поддержкой кириллицы (UTF-8)
- Рекурсивный парсер (приоритеты операторов, аннотации)
- IR-генератор с типами и владением
- LLVM-бэкенд (≈19× быстрее ASM)
- Самокомпиляция: fixed point (`selfc == selfc`)

#### Язык

- VSO-синтаксис (Глагол–Субъект–Объект)
- Операторы потока: `>>`, `>>>`, `||`, `!`, `~`
- 64 корня в 5 категориях
- 10 типов шаблонов через гласные (А–Я)
- Диапазоны `0..N`
- Комплементарные корни (СПР ↔ РПС)
- Аннотации `@пластичность`, `@comptime`, `@ускорить`, `@кэшировать`,
  `@шифровать`, `@параллелить`, `@сон`, `@тест`
- Специализация глаголов по типам (R3)
- Типы: `Нечто`/`Ничто`, `Успех`/`Ошибка`, TTL, единицы измерения
- Потоковое программирование (`поток`)
- Агентное программирование (`агент`)
- CLI: `zipc запустить` / `zipc тест` / `zipc собрать` (R7)

#### Экосистема

- `зипм` — пакетный менеджер
- `зип-lsp` — Language Server Protocol
- VSCode-расширение (подсветка + LSP + отладка)
- `зип-отладчик` — DAP-адаптер
- `зип-профиль` — профилировщик (Callgrind + Flamegraph)
- Стандартная библиотека (`zip-std/`)
  - Ядро
  - Тензоры (GEMM ≈4.2× от NumPy)
  - Нейросети (XOR обучен)
  - Сеть (HTTP/HTTPS)
  - Криптография (SHA-256, AES-256-GCM)
  - TTL, единицы измерения, потоки, агенты

#### Производительность

- LLVM-бэкенд: ≈19× быстрее ASM
- JIT-компиляция: прирост на горячих путях (MVP)
- GEMM 512×512: ≈4.2× от NumPy
- Нулевые утечки памяти (Move-семантика)

#### Защита

- Постквантовая криптография (LWE-шум)
- Анализ владения (Polonius-стиль)
- Ошибки с восстановлением пути передачи

#### Пакетная статистика

- Тестов: 200+
- Файлов: 200+
- Размер компилятора: ~5 МБ
- Размер STD: ~100 КБ
