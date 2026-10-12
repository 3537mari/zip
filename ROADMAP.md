# Дорожная карта ZIP

## v1.0.0 (2026-10-11) ✅

Первый стабильный релиз

## v1.1.0 (2026-10-11) ✅

Документация и производительность

## v1.2.0 (2026-10-11) ✅

Расширение STD + плагины + сайт

## v2.0.0 (2026-10-11) ✅

Полная независимость от LLVM (ZIPB, VM v3, компилятор на ZIP, верификация)

## v2.1.0 (2026-10-11) ✅ ЗАВЕРШЁН

- [x] Профилирование компилятора ✅
- [x] Расширение тестов до 90% (ядро) ✅
- [x] Бенчмарки vs Python/Rust/Go ✅
- [x] Реальный проект: веб-сервис (TODO REST API) ✅

**Дата релиза:** 2026-10-11

## v2.2.0 (2026-10-11) — Сообщество — ✅ ЗАВЕРШЁН

- [x] Публикации (готовы тексты) — [`publishing/`](publishing/) · [`docs/ПУБЛИКАЦИИ.md`](docs/ПУБЛИКАЦИИ.md)
- [x] Видео-туториалы (сценарии) — [`learning/видео/`](learning/видео/)
- [x] Интерактивный курс — [`веб-сайт/курс/`](веб-сайт/курс/) → https://3537mari.github.io/zip/курс/
- [x] Социальные материалы — [`learning/соцсети/`](learning/соцсети/)
- [x] Онлайн-курс опубликован ✅
- [x] Сообщество (Discord / Telegram / Discussions) — [`community/`](community/)
- [x] CONTRIBUTING / CODE_OF_CONDUCT / GOVERNANCE / FAQ ✅
- [ ] Публикация в интернете (HN / Reddit / Habr / YouTube) — тексты готовы
- [ ] Видео записаны — сценарии и план готовы
- [ ] Покрытие всего crate zipc ≥90% (бэклог)

**Дата релиза:** 2026-10-11

## v3.0.0 (2026-10-11) — ✅ ЗАВЕРШЁН

- [x] Bootstrap на C (~1650+ строк, расширение к ~5000) ✅
- [x] Fixed-point самокомпиляция (детерминизм C-bootstrap) ✅
- [x] Убрать LLVM из основного пути ✅
- [x] Installers (4 платформы) ✅
- [x] Offline installer ✅
- [ ] JIT v3 (бэклог)
- [ ] Homebrew / Chocolatey (бэклог)

**Дата релиза:** 2026-10-11

## v4.0.0 (2026-10-12) — ✅ Runtime / STD / selfc-конвейер

- [x] Runtime на ZIP (syscalls) ✅ — [`docs/RUNTIME-НА-ZIP.md`](docs/RUNTIME-НА-ZIP.md)
- [x] Свой аллокатор / вывод / файлы ✅
- [x] Ядро / тензоры / нейросети / крипто на ZIP ✅
- [x] JSON/CSV/RegEx на ZIP (MVP) ✅
- [x] Inline-asm syscalls (x86-64 Linux) ✅
- [x] Лексер / парсер / генератор API на ZIP ✅ — [`zip-self-zip/`](zip-self-zip/)
- [x] Fixed-point самокомпиляция (`главное.zip`) ✅
- [ ] Сеть на ZIP (полный HTTP/TLS)
- [ ] STD без VM builtins (100% lowering)
- [ ] Bootstrap-алгоритмы целиком на ZIP (без C на stage 1+)
- [ ] VM на ZIP (полная) — [`zip-vm-zip/`](zip-vm-zip/)
- [ ] Inline-asm Windows/macOS

См. [docs/ОТЧЁТ-v4.md](docs/ОТЧЁТ-v4.md), [ROADMAP_V4.md](ROADMAP_V4.md)

## v5.0.0 (2028) — план

- [ ] Полный порт lex/parse/codegen на ZIP
- [ ] Кросс-компиляция (ARM, RISC-V, WASM)
- [ ] Мобильные биндинги
- [ ] GPU-вычисления
- [ ] Пакетный реестр
- [ ] ИИ-советник 2.0

См. [ROADMAP_V5.md](ROADMAP_V5.md). Подробности: [МАСТЕР-ПЛАН.md](МАСТЕР-ПЛАН.md).
