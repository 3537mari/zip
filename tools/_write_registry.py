# -*- coding: utf-8 -*-
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
reg = ROOT / "реестр"
reg.mkdir(parents=True, exist_ok=True)

(reg / "README.md").write_text(
    """# Реестр пакетов ZIP

Локальный реестр для `зипм`.

## Структура

```
реестр/
├── index.toml
├── пакеты/<имя>/<версия>/
│   ├── зипм.манифест
│   ├── src/
│   └── checksum.sha256
└── сервер/
```

Переменная `ZIPM_REGISTRY` / `ЗИПМ_РЕЕСТР` указывает на этот каталог или URL.
""",
    encoding="utf-8",
)

(reg / "index.toml").write_text(
    """# Индекс пакетов ZIP

[пакеты.тензоры]
версии = ["1.0.0"]
описание = "Тензорные операции STD"

[пакеты.нейросети]
версии = ["0.5.0"]
описание = "Нейросети и обучение"
""",
    encoding="utf-8",
)

t = reg / "пакеты" / "тензоры" / "1.0.0"
t.mkdir(parents=True, exist_ok=True)
(t / "зипм.манифест").write_text(
    """имя = "тензоры"
версия = "1.0.0"
автор = "ZIP STD"
лицензия = "MIT"
описание = "Тензорные операции"

[сборка]
тип = "библиотека"
точка_входа = "src/тензоры.zip"
""",
    encoding="utf-8",
)
(t / "src").mkdir(exist_ok=True)
(t / "src" / "тензоры.zip").write_text("// библиотека тензоров (заглушка реестра)\n", encoding="utf-8")
(t / "checksum.sha256").write_text(("0" * 64) + "  зипм.манифест\n", encoding="utf-8")

n = reg / "пакеты" / "нейросети" / "0.5.0"
n.mkdir(parents=True, exist_ok=True)
(n / "зипм.манифест").write_text(
    """имя = "нейросети"
версия = "0.5.0"
автор = "ZIP STD"
лицензия = "MIT"
описание = "Нейросети"

[сборка]
тип = "библиотека"
точка_входа = "src/нейросети.zip"
""",
    encoding="utf-8",
)
(n / "src").mkdir(exist_ok=True)
(n / "src" / "нейросети.zip").write_text("// нейросети (заглушка)\n", encoding="utf-8")
(n / "checksum.sha256").write_text(("0" * 64) + "  зипм.манифест\n", encoding="utf-8")

srv = reg / "сервер"
srv.mkdir(parents=True, exist_ok=True)
(srv / "сервер.zip").write_text(
    """// HTTP-сервер реестра пакетов (заготовка)
#runtime("http_client")

печать_строки("Реестр ZIP (заготовка). Укажите ZIPM_REGISTRY на каталог реестр/")
""",
    encoding="utf-8",
)
(srv / "зипм.манифест").write_text(
    """имя = "реестр-сервер"
версия = "0.1.0"
лицензия = "MIT"
описание = "Локальный HTTP-сервер реестра"

[сборка]
тип = "программа"
точка_входа = "сервер.zip"
выход = "цель/сервер"
""",
    encoding="utf-8",
)
print("registry ok", reg)
