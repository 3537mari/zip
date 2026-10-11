#!/usr/bin/env python3
"""Проверка документации ZIP на консистентность (кроссплатформенно)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

ОБЯЗАТЕЛЬНЫЕ = [
    "README.md",
    "CHANGELOG.md",
    "ROADMAP.md",
    "МАСТЕР-ПЛАН.md",
    "docs/УЧЕБНИК.md",
    "docs/СПРАВОЧНИК.md",
    "docs/STD.md",
    "docs/COOKBOOK.md",
    "docs/FAQ.md",
    "docs/МИГРАЦИЯ.md",
    "docs/ИНДЕКС.md",
]

LINK_RE = re.compile(r"\]\(([^)]+\.md)(#[^)]*)?\)")


def main() -> int:
    print("=== Проверка документации ZIP ===\n")

    print("1. Проверка обязательных файлов:")
    missing = 0
    for rel in ОБЯЗАТЕЛЬНЫЕ:
        p = ROOT / rel
        if p.is_file():
            print(f"  ✓ {rel}")
        else:
            print(f"  ❌ {rel} — ОТСУТСТВУЕТ")
            missing += 1
    print()

    print("2. Проверка ссылок:")
    broken = 0
    files = list((ROOT / "docs").glob("*.md")) + [
        ROOT / "README.md",
        ROOT / "CHANGELOG.md",
        ROOT / "ROADMAP.md",
        ROOT / "МАСТЕР-ПЛАН.md",
    ]
    for path in files:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for m in LINK_RE.finditer(text):
            rel = m.group(1)
            if rel.startswith("http"):
                continue
            candidates = [
                (path.parent / rel).resolve(),
                (ROOT / rel).resolve(),
                (ROOT / "docs" / Path(rel).name).resolve(),
            ]
            if not any(c.is_file() for c in candidates):
                try:
                    shown = path.relative_to(ROOT)
                except ValueError:
                    shown = path
                print(f"  ❌ {shown} → {rel}")
                broken += 1
    if broken == 0:
        print("  ✓ Битых ссылок нет")
    print()

    print("3. Размер документации:")
    size = sum(p.stat().st_size for p in (ROOT / "docs").rglob("*") if p.is_file())
    print(f"  {size / 1024:.1f} KiB\tdocs/")
    print()

    print("4. Строк в документации:")
    lines = sum(
        len(p.read_text(encoding="utf-8").splitlines())
        for p in (ROOT / "docs").glob("*.md")
    )
    print(f"  {lines} total (docs/*.md)")
    print()

    print("5. Проверка UTF-8:")
    utf_ok = True
    for p in files:
        if not p.is_file():
            continue
        try:
            p.read_text(encoding="utf-8")
        except Exception as e:
            print(f"  ❌ {p.relative_to(ROOT)} — {e}")
            utf_ok = False
    if utf_ok:
        print("  ✓ Все файлы в UTF-8")
    print()

    print("=== Проверка завершена ===")
    return 1 if missing or broken or not utf_ok else 0


if __name__ == "__main__":
    sys.exit(main())
