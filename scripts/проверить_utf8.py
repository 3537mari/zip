#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Проверяет все .zip файлы на корректный UTF-8."""

from pathlib import Path
import sys
import re

корень = Path(__file__).resolve().parent.parent

# Паттерны, в которых латиница легальна (строки — до //, иначе http:// режется)
ЛЕГАЛЬНАЯ_ЛАТИНИЦА = [
    re.compile(r'"[^"]*"'),  # строковый литерал
    re.compile(r"#link\([^)]*\)"),  # FFI #link
    re.compile(r"#runtime\([^)]*\)"),  # FFI #runtime
    re.compile(r"#include\([^)]*\)"),  # #include
    re.compile(r"/\*.*?\*/", re.DOTALL),  # комментарий /* */
    re.compile(r"//.*$"),  # комментарий //
]

# Известные латинские FFI-идентификаторы (после очистки #link)
ЛЕГАЛЬНЫЕ_ИДЕНТ = re.compile(
    r"\b(sqrt|sin|cos|exp|malloc|free|fopen|fclose|fread|fwrite|time|"
    r"socket|connect|send|recv|close|aligned_alloc|SHA256|"
    r"http_get|http_тело|zip_http_get|zip_parse_url|"
    r"url|host|path|port|AF_INET|SOCK_STREAM|GET|HTTP|"
    r"example|com)\b"
)


def очистить_от_легальной_латиницы(строка: str) -> str:
    """Удаляет все фрагменты, где латиница допустима."""
    for паттерн in ЛЕГАЛЬНАЯ_ЛАТИНИЦА:
        строка = паттерн.sub("", строка)
    строка = ЛЕГАЛЬНЫЕ_ИДЕНТ.sub("", строка)
    return строка


def счёт_кириллицы(текст: str) -> int:
    return sum(1 for ch in текст if "\u0400" <= ch <= "\u04FF")


ОШИБКИ = []
ПРЕДУПРЕЖДЕНИЯ = []
ХОРОШИЕ = []

for путь in sorted(корень.rglob("*.zip")):
    rel = str(путь.relative_to(корень)).replace("\\", "/")
    if "target" in rel.split("/") or ".git" in rel.split("/"):
        continue
    try:
        сырые = путь.read_bytes()
        if сырые.startswith(b"\xef\xbb\xbf"):
            ОШИБКИ.append(f"{rel} — UTF-8 с BOM (нужен без BOM)")
            continue
        содержимое = сырые.decode("utf-8")
        if any("\u0400" <= ch <= "\u04FF" for ch in путь.name) and счёт_кириллицы(содержимое) < 5:
            ОШИБКИ.append(
                f"{rel} — имя кириллическое, но в файле почти нет кириллицы (повреждён?)"
            )
            continue

        ХОРОШИЕ.append(rel)

        for i, строка_исх in enumerate(содержимое.splitlines(), 1):
            stripped = строка_исх.strip()
            if stripped.startswith("печать_") or stripped.startswith("Печать"):
                continue
            строка_очищ = очистить_от_легальной_латиницы(строка_исх)
            for символ in строка_очищ:
                if символ.isascii() and символ.isalpha():
                    ОШИБКИ.append(
                        f"{rel}:{i} — латиница '{символ}' вне комментария: {stripped[:60]}"
                    )
                    break
    except UnicodeDecodeError as e:
        ОШИБКИ.append(f"{rel} — повреждён UTF-8: {e}")

print(f"Хороших файлов: {len(ХОРОШИЕ)}")
print(f"Предупреждений: {len(ПРЕДУПРЕЖДЕНИЯ)}")
print(f"Ошибок: {len(ОШИБКИ)}")
for ошибка in ОШИБКИ:
    print(f"  {ошибка}")

sys.exit(1 if ОШИБКИ else 0)
