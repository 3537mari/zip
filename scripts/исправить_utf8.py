#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Пересохраняет .zip исходники в UTF-8 без BOM (безопасно)."""

from pathlib import Path

корень = Path(__file__).resolve().parent.parent


def счёт_кириллицы(текст: str) -> int:
    return sum(1 for ch in текст if "\u0400" <= ch <= "\u04FF")


def похоже_на_кракозябры(текст: str) -> bool:
    """Мусор вроде DC=:F8O вместо «функция» (cp866, прочитанный как Latin)."""
    маркеры = ("DC=:F8O", "B8?", "40==K5", "2>72@0B")
    return any(м in текст for м in маркеры)


исправлено = 0
пропущено = 0

for путь in sorted(корень.rglob("*.zip")):
    rel = str(путь.relative_to(корень)).replace("\\", "/")
    if "target" in rel.split("/") or ".git" in rel.split("/"):
        continue

    сырые = путь.read_bytes()

    # 1) Валидный UTF-8 — только нормализация BOM/CRLF
    try:
        текст = сырые.decode("utf-8")
        if похоже_на_кракозябры(текст):
            # UTF-8 байты содержат уже испорченный текст — восстановить из однобайтовых
            raise UnicodeDecodeError("utf-8", сырые, 0, 1, "mojibake")
        if сырые.startswith(b"\xef\xbb\xbf"):
            текст = текст.lstrip("\ufeff")
        нормализовано = текст.replace("\r\n", "\n").replace("\r", "\n")
        новые = нормализовано.encode("utf-8")
        if новые != сырые:
            print(f"Нормализую {rel} (BOM/CRLF -> UTF-8 LF)")
            путь.write_bytes(новые)
            исправлено += 1
        else:
            пропущено += 1
        continue
    except UnicodeDecodeError:
        pass

    # 2) Не UTF-8 или кракозябры — пробуем cp866/cp1251/koi8-r
    содержимое = None
    исходная = None
    for кодировка in ("cp866", "cp1251", "koi8-r"):
        try:
            кандидат = сырые.decode(кодировка)
        except UnicodeDecodeError:
            continue
        if счёт_кириллицы(кандидат) < 10:
            continue
        if похоже_на_кракозябры(кандидат):
            continue
        содержимое = кандидат
        исходная = кодировка
        break

    if содержимое is None:
        print(f"Не могу исправить: {rel} (запустите: python scripts/restore_std_utf8.py)")
        continue

    нормализовано = содержимое.replace("\r\n", "\n").replace("\r", "\n")
    print(f"Исправляю {rel} из {исходная} -> UTF-8")
    путь.write_bytes(нормализовано.encode("utf-8"))
    исправлено += 1

print(f"Готово. Исправлено: {исправлено}, без изменений: {пропущено}.")
