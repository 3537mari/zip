# зип-отладчик

DAP-адаптер для языка ZIP (Debug Adapter Protocol).

## Сборка

```bash
cargo build --release
```

Бинарники: `target/release/zip-debugger` и `target/release/зип-отладчик`.

## Запуск

Клиент (VS Code / Cursor) запускает адаптер по stdio. Обёртка расширения: `vscode-zip/out/debugAdapter.js`.

Конфигурация launch:

```json
{
  "type": "zip",
  "request": "launch",
  "name": "Отладка ZIP",
  "программа": "${workspaceFolder}/examples/отладка_тест.zip"
}
```

## Возможности (MVP)

- точки останова по строкам
- continue / next / stepIn / stepOut
- стек вызовов, локальные переменные
- evaluate по именам переменных
- остановка на исключении (деление на ноль)

Интерпретация — пошаговый обход AST (без карты строк LLVM).
