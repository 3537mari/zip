# VSCode / Cursor — поддержка ZIP

Подсветка синтаксиса (TextMate) + Language Server Protocol (`зип-lsp` / `zip-lsp`).

## Установка

1. Соберите LSP:

```bash
cargo build --release --manifest-path зип-lsp/Cargo.toml --target-dir зип-lsp/target
```

2. Установите зависимости расширения:

```bash
cd vscode-zip
npm install
```

3. В Cursor/VSCode: **Extensions → … → Install from VSIX…** после:

```bash
npx vsce package --no-dependencies
```

Или откройте папку `vscode-zip` через **Developer: Install Extension from Location…**.

## Настройка

- `zip.lsp.path` — полный путь к `zip-lsp.exe` (по умолчанию автопоиск в `../зип-lsp/target/release/`).

## Возможности

- Подсветка: ключевые слова, глаголы, операторы `>>` `||` `!` `~`, корни, шаблоны
- Автодополнение (Ctrl+Space)
- Hover-документация
- Диагностика лексики/парсера
- Document symbols / semantic tokens

## Примечание про `.zip`

Расширение `.zip` также используется архивами. Для исходников ZIP удобнее ассоциация «Open With → ZIP» или расширение `.зип`.
