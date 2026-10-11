# -*- coding: utf-8 -*-
"""Сводка результатов бенчмарков → markdown + данные для графиков."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RESULTS_JSON = ROOT / "results.json"
RESULTS_MD = ROOT / "результаты.md"
if not RESULTS_JSON.exists() and (ROOT / "результаты.json").exists():
    RESULTS_JSON = ROOT / "результаты.json"
DOCS = ROOT.parent / "docs"


def load() -> dict:
    if RESULTS_JSON.exists():
        return json.loads(RESULTS_JSON.read_text(encoding="utf-8"))
    return {}


def fmt(v):
    if v is None:
        return "—"
    if isinstance(v, (int, float)):
        return f"{v:.2f}"
    return str(v)


def relative(base, val):
    if not base or not val or base <= 0:
        return "—"
    return f"{val / base:.2f}×"


def main():
    data = load()
    tasks = ["fibonacci", "gemm", "json", "strings", "http", "nn"]
    langs = ["ZIP", "Python", "Rust", "Go"]

    lines = [
        "# Результаты бенчмарков",
        "",
        f"**Источник:** `{RESULTS_JSON.name}`",
        "",
    ]

    for task in tasks:
        lines.append(f"## {task}")
        lines.append("")
        lines.append("| Язык | Время (ms) | Размер exe (байт) |")
        lines.append("|------|------------|------------------|")
        block = data.get(task, {})
        for lang in langs:
            row = block.get(lang, {})
            lines.append(
                f"| {lang} | {fmt(row.get('ms'))} | {fmt(row.get('size'))} |"
            )
        lines.append("")

    RESULTS_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(RESULTS_MD.read_text(encoding="utf-8"))


if __name__ == "__main__":
    main()
