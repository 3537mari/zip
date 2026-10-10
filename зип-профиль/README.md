# зип-профиль

Инструментированный профилировщик для ZIP (callgrind-стиль) с экспортом JSON и flamegraph (inferno).

## Сборка

```bash
cargo build --release
```

## Пример

```bash
./target/release/zip-profile.exe ../examples/медленная.zip --топ 10 --тихо
./target/release/zip-profile.exe ../examples/медленная.zip --json profile.json --svg flamegraph --тихо
```

## Типы

- `cpu` — exclusive/inclusive время и стеки (по умолчанию)
- `память` / `аллокации` — дополнительно считает аллокации кадров
- `io` — зарезервировано (пока как cpu)

VS Code: задача «Профилировать ZIP» (`.vscode/tasks.json`).
