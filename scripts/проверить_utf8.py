#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Проверяет все .zip файлы на корректный UTF-8.

Коды выхода:
  0 — все файлы валидный UTF-8 без BOM
  1 — есть ошибки кодировки / BOM / пустая кириллица в STD

Латиница вне комментариев/директив/строк — предупреждение
(FFI: sqrt, malloc; ASCII-алиасы путей), не валит проверку по умолчанию.
Передайте --strict-latin чтобы сделать предупреждения ошибками.
"""

from pathlib import Path
import sys

корень = Path(__file__).resolve().parent.parent
strict_latin = "--strict-latin" in sys.argv

ОШИБКИ = []
ПРЕДУПРЕЖДЕНИЯ = []
ХОРОШИЕ = []


def в_строковом_литерале(строка: str, индекс: int) -> bool:
    в_строке = False
    экранирован = False
    for i, с in enumerate(строка):
        if i >= индекс:
            return в_строке
        if экранирован:
            экранирован = False
            continue
        if с == "\\" and в_строке:
            экранирован = True
            continue
        if с == '"':
            в_строке = not в_строке
    return в_строке


def счёт_кириллицы(текст: str) -> int:
    return sum(1 for ch in текст if "\u0400" <= ch <= "\u04FF")


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
        кир = счёт_кириллицы(содержимое)

        # STD/examples с кириллицей в имени должны содержать кириллицу в теле
        имя_имеет_кир = any("\u0400" <= ch <= "\u04FF" for ch in путь.name)
        if имя_имеет_кир and кир < 5:
            ОШИБКИ.append(
                f"{rel} — имя кириллическое, но в файле почти нет кириллицы (повреждён?)"
            )
            continue

        ХОРОШИЕ.append(rel)
        for i, строка in enumerate(содержимое.splitlines(), 1):
            код = строка
            if "//" in код:
                код = код[: код.index("//")]
            if "#" in код:
                continue
            for j, символ in enumerate(код):
                if символ.isascii() and символ.isalpha():
                    if в_строковом_литерале(код, j):
                        continue
                    сообщение = f"{rel}:{i} — латиница '{символ}' вне комментария"
                    if strict_latin:
                        ОШИБКИ.append(сообщение)
                    else:
                        ПРЕДУПРЕЖДЕНИЯ.append(сообщение)
                    break
    except UnicodeDecodeError as e:
        ОШИБКИ.append(f"{rel} — повреждён UTF-8: {e}")

print(f"Хороших файлов: {len(ХОРОШИЕ)}")
print(f"Предупреждений (латиница): {len(ПРЕДУПРЕЖДЕНИЯ)}")
print(f"Ошибок: {len(ОШИБКИ)}")
for п in ПРЕДУПРЕЖДЕНИЯ[:40]:
    print(f"  ~ {п}")
if len(ПРЕДУПРЕЖДЕНИЯ) > 40:
    print(f"  ~ ... ещё {len(ПРЕДУПРЕЖДЕНИЯ) - 40}")
for ошибка in ОШИБКИ:
    print(f"  ! {ошибка}")

sys.exit(1 if ОШИБКИ else 0)
