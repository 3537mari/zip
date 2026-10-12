# ZIP VM на ZIP (v4, MVP)

Интерпретатор `.zipb` v3 на языке ZIP. Сейчас — каркас; полный цикл исполнения — в Rust `zip-vm`.

## Структура

```
zip-vm-zip/
├── README.md
└── src/
    └── главная.zip   — точка входа (MVP: вызов zip-vm Rust через #runtime)
```

## Сборка MVP

```bash
# Заготовка runtime (clang)
zipc собрать zip-vm-zip/src/главная.zip -o zip-vm-zip-mvp
```

## Дорожная карта

1. Декодер ZIPB на ZIP
2. Стек / кадры на ZIP
3. Исполнитель опкодов
4. Fixed-point: zip-vm-zip исполняет сам себя
