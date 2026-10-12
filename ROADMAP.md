# Дорожная карта ZIP

## v1.0.0 – v3.0.0 ✅

См. [CHANGELOG.md](CHANGELOG.md).

## v4.0.0 (2026-10-12) — ✅ Runtime / STD / selfc-конвейер

- Runtime / STD на ZIP, fixed-point selfc (через C-движок stage-0)
- См. [docs/ОТЧЁТ-v4.md](docs/ОТЧЁТ-v4.md)

## v5.0.0 (2026-10-12) — ✅ Конвейер в VM

- [x] Лексер в VM ✅
- [x] Парсер в VM ✅
- [x] Генератор ZIPB в VM ✅
- [x] Полная цепочка + CLI `--gen` ✅
- [x] Тесты цепочки ✅

**Дата:** 2026-10-12 · [docs/ОТЧЁТ-v5.md](docs/ОТЧЁТ-v5.md) · [ROADMAP_V5.md](ROADMAP_V5.md)

## v6.0.0 (2028) — план

### Язык и lowering

- [x] `структура` — парсинг MVP ✅
- [ ] Литералы / поля / массивы / указатели
- [ ] Лексер / парсер / генератор на чистом ZIP
- [ ] Убрать C-bootstrap на stage 1+

### Платформы

- [ ] Кросс-компиляция, мобильные, GPU, реестр

См. [ROADMAP_V6.md](ROADMAP_V6.md).
