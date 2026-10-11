# -*- coding: utf-8 -*-
"""Генерация docs/БЕНЧМАРКИ.md из результаты.json."""
from __future__ import annotations

import json
import platform
import subprocess
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parent
_json = ROOT / "results.json"
if not _json.exists():
    _json = ROOT / "результаты.json"
DATA = json.loads(_json.read_text(encoding="utf-8"))
OUT = REPO / "docs" / "БЕНЧМАРКИ.md"


def ver(cmd):
    try:
        return subprocess.check_output(cmd, text=True, stderr=subprocess.STDOUT).strip().splitlines()[0]
    except Exception:
        return "n/a"


def ms(task, lang):
    v = DATA.get(task, {}).get(lang, {}).get("ms")
    return v


def rel(task, lang, base_lang):
    a = ms(task, lang)
    b = ms(task, base_lang)
    if not a or not b or b == 0:
        return "—"
    return f"{a / b:.2f}×"


def cell(task, lang):
    v = ms(task, lang)
    if v is None:
        return "—"
    return f"{v:.2f}"


def main():
    rust_v = ver(["rustc", "--version"])
    py_v = ver(["python", "--version"])
    go_v = ver(["go", "version"])
    clang_v = ver(["clang", "--version"])

    tasks = ["fibonacci", "gemm", "json", "strings", "http", "nn"]
    # среднее ускорение vs Python (меньше ms = быстрее)
    speedups = []
    for t in tasks:
        p = ms(t, "Python")
        z = ms(t, "ZIP")
        if p and z and z > 0:
            speedups.append(p / z)
    avg_vs_py = sum(speedups) / len(speedups) if speedups else None

    rust_sp = []
    for t in tasks:
        r = ms(t, "Rust")
        z = ms(t, "ZIP")
        if r and z and r > 0:
            rust_sp.append(z / r)
    avg_vs_rust = sum(rust_sp) / len(rust_sp) if rust_sp else None

    md = f"""# Бенчмарки ZIP vs Python vs Rust vs Go

## Методология

- **Оборудование:** {platform.node()} / {platform.processor() or platform.machine()}
- **ОС:** {platform.system()} {platform.release()}
- **Дата:** {date.today().isoformat()}
- **ZIP:** v2.1.0-dev (`zipc` release; Fibonacci — native backend; GEMM/JSON/strings/http/nn — STD `#runtime` + clang -O3)
- **Python:** {py_v} (GEMM — NumPy)
- **Rust:** {rust_v} (`-O` / `--release`)
- **Go:** {go_v}
- **Clang:** {clang_v}

Тайминг: внутренний замер программы (ms), если напечатан; иначе wall-clock процесса.
Исходники: [`benchmarks/`](../benchmarks/).

## Результаты

### 1. Fibonacci(40) — рекурсия

| Язык | Время (ms) | Относительно Rust |
|------|------------|-------------------|
| **ZIP** | {cell('fibonacci','ZIP')} | {rel('fibonacci','ZIP','Rust')} |
| **Rust** | {cell('fibonacci','Rust')} | 1× |
| **Go** | {cell('fibonacci','Go')} | {rel('fibonacci','Go','Rust')} |
| **Python** | {cell('fibonacci','Python')} | {rel('fibonacci','Python','Rust')} |

### 2. GEMM 512×512 — матричное умножение

| Язык | Время (ms) | Относительно NumPy |
|------|------------|---------------------|
| **ZIP** (STD runtime / clang) | {cell('gemm','ZIP')} | {rel('gemm','ZIP','Python')} |
| **NumPy (Python)** | {cell('gemm','Python')} | 1× |
| **Rust** (naive triple loop) | {cell('gemm','Rust')} | {rel('gemm','Rust','Python')} |
| **Go** (naive triple loop) | {cell('gemm','Go')} | {rel('gemm','Go','Python')} |

> ZIP GEMM — оптимизированный blocked GEMM из `zip-std/runtime/gemm.c`. Rust/Go здесь намеренно naive; NumPy использует BLAS.

### 3. JSON-парсинг (10 000 объектов)

| Язык | Время (ms) | Относительно Rust |
|------|------------|-------------------|
| **ZIP** | {cell('json','ZIP')} | {rel('json','ZIP','Rust')} |
| **Python** | {cell('json','Python')} | {rel('json','Python','Rust')} |
| **Rust (serde_json)** | {cell('json','Rust')} | 1× |
| **Go (encoding/json)** | {cell('json','Go')} | {rel('json','Go','Rust')} |

### 4. Строки (1 000 000 операций)

| Язык | Время (ms) | Относительно Rust |
|------|------------|-------------------|
| **ZIP** | {cell('strings','ZIP')} | {rel('strings','ZIP','Rust')} |
| **Python** | {cell('strings','Python')} | {rel('strings','Python','Rust')} |
| **Rust** | {cell('strings','Rust')} | 1× |
| **Go** | {cell('strings','Go')} | {rel('strings','Go','Rust')} |

### 5. HTTP — 5 000 000 разборов GET

| Язык | Время (ms) | Относительно Rust |
|------|------------|-------------------|
| **ZIP** | {cell('http','ZIP')} | {rel('http','ZIP','Rust')} |
| **Python** | {cell('http','Python')} | {rel('http','Python','Rust')} |
| **Rust** | {cell('http','Rust')} | 1× |
| **Go** | {cell('http','Go')} | {rel('http','Go','Rust')} |

Микробенч разбора запроса (без сети) — сопоставимая нагрузка на всех языках.

### 6. Нейросеть XOR — 10 000 эпох

| Язык | Время (ms) | Относительно Rust |
|------|------------|-------------------|
| **ZIP** | {cell('nn','ZIP')} | {rel('nn','ZIP','Rust')} |
| **Python** | {cell('nn','Python')} | {rel('nn','Python','Rust')} |
| **Rust** | {cell('nn','Rust')} | 1× |
| **Go** | {cell('nn','Go')} | {rel('nn','Go','Rust')} |

## Сводная таблица (ms)

| Задача | ZIP | Python | Rust | Go |
|--------|-----|--------|------|-----|
| Fibonacci | {cell('fibonacci','ZIP')} | {cell('fibonacci','Python')} | {cell('fibonacci','Rust')} | {cell('fibonacci','Go')} |
| GEMM | {cell('gemm','ZIP')} | {cell('gemm','Python')} | {cell('gemm','Rust')} | {cell('gemm','Go')} |
| JSON | {cell('json','ZIP')} | {cell('json','Python')} | {cell('json','Rust')} | {cell('json','Go')} |
| Strings | {cell('strings','ZIP')} | {cell('strings','Python')} | {cell('strings','Rust')} | {cell('strings','Go')} |
| HTTP | {cell('http','ZIP')} | {cell('http','Python')} | {cell('http','Rust')} | {cell('http','Go')} |
| NN XOR | {cell('nn','ZIP')} | {cell('nn','Python')} | {cell('nn','Rust')} | {cell('nn','Go')} |

## Выводы

- ZIP vs Python (среднее отношение времён Python/ZIP по задачам): **{f'{avg_vs_py:.1f}×' if avg_vs_py else '—'}**
- ZIP vs Rust (среднее ZIP/Rust): **{f'{avg_vs_rust:.2f}×' if avg_vs_rust else '—'}** (1.0 = паритет)
- На рекурсии (Fibonacci) ZIP native близок к компилируемым языкам и существенно быстрее CPython.
- На GEMM экосистема ZIP опирается на clang -O3 runtime и конкурирует с NumPy/BLAS-уровнем; naive Rust/Go медленнее.
- JSON/strings/http/nn: ZIP STD runtime на C показывает производительность уровня системных языков.

## Графики

![Производительность](графики/производительность.svg)

## Размеры бинарников (байт)

| Задача | ZIP | Rust | Go |
|--------|-----|------|-----|
"""
    for t in tasks:
        z = DATA.get(t, {}).get("ZIP", {}).get("size", "—")
        r = DATA.get(t, {}).get("Rust", {}).get("size", "—")
        g = DATA.get(t, {}).get("Go", {}).get("size", "—")
        md += f"| {t} | {z} | {r} | {g} |\n"

    md += """
---
*Сгенерировано `benchmarks/scripts/сгенерировать_отчёт.py`.*
"""
    OUT.write_text(md, encoding="utf-8")
    print("wrote", OUT)


if __name__ == "__main__":
    main()
