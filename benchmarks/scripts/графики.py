# -*- coding: utf-8 -*-
"""Генерация SVG/PNG графиков из результаты.json."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS_GRAPH = ROOT.parent / "docs" / "графики"
RESULTS_JSON = ROOT / "results.json"
if not RESULTS_JSON.exists() and (ROOT / "результаты.json").exists():
    RESULTS_JSON = ROOT / "результаты.json"


def main():
    DOCS_GRAPH.mkdir(parents=True, exist_ok=True)
    data = json.loads(RESULTS_JSON.read_text(encoding="utf-8"))
    langs = ["ZIP", "Python", "Rust", "Go"]
    tasks = ["fibonacci", "gemm", "json", "strings", "http", "nn"]

    try:
        import matplotlib.pyplot as plt
        import numpy as np
    except ImportError:
        # SVG без matplotlib
        write_plain_svg(data, langs, tasks)
        return

    x = np.arange(len(langs))
    width = 0.13
    fig, ax = plt.subplots(figsize=(12, 6))
    for i, task in enumerate(tasks):
        vals = []
        for lang in langs:
            ms = data.get(task, {}).get(lang, {}).get("ms")
            vals.append(ms if ms and ms > 0 else float("nan"))
        ax.bar(x + (i - 2.5) * width, vals, width, label=task)

    ax.set_ylabel("Время (ms)")
    ax.set_title("Производительность ZIP vs Python vs Rust vs Go")
    ax.set_xticks(x)
    ax.set_xticklabels(langs)
    ax.legend(fontsize=8)
    ax.set_yscale("log")
    ax.grid(True, axis="y", alpha=0.3)
    out = DOCS_GRAPH / "производительность.svg"
    fig.tight_layout()
    fig.savefig(out)
    fig.savefig(DOCS_GRAPH / "производительность.png", dpi=140)
    print("wrote", out)


def write_plain_svg(data, langs, tasks):
    # простой бар-чарт вручную
    w, h = 900, 480
    margin = 60
    bars = []
    max_v = 1.0
    for t in tasks:
        for lang in langs:
            v = data.get(t, {}).get(lang, {}).get("ms") or 0
            if v > max_v:
                max_v = v
    import math

    max_v = max(max_v, 1.0)
    n = len(langs)
    group_w = (w - 2 * margin) / n
    colors = ["#2563eb", "#16a34a", "#ca8a04", "#dc2626", "#9333ea", "#0891b2"]
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">',
        f'<rect width="100%" height="100%" fill="#fafafa"/>',
        f'<text x="{w/2}" y="28" text-anchor="middle" font-size="18" font-family="sans-serif">'
        f"ZIP vs Python vs Rust vs Go</text>",
    ]
    for gi, lang in enumerate(langs):
        gx = margin + gi * group_w
        for ti, task in enumerate(tasks):
            v = data.get(task, {}).get(lang, {}).get("ms") or 0
            # log scale height
            ratio = math.log10(v + 1) / math.log10(max_v + 1)
            bh = max(2, ratio * (h - 100))
            bw = group_w / (len(tasks) + 1)
            bx = gx + ti * bw
            by = h - 40 - bh
            parts.append(
                f'<rect x="{bx:.1f}" y="{by:.1f}" width="{bw*0.9:.1f}" height="{bh:.1f}" '
                f'fill="{colors[ti % len(colors)]}"/>'
            )
        parts.append(
            f'<text x="{gx + group_w/2}" y="{h-16}" text-anchor="middle" '
            f'font-size="14" font-family="sans-serif">{lang}</text>'
        )
    parts.append("</svg>")
    out = DOCS_GRAPH / "производительность.svg"
    out.write_text("\n".join(parts), encoding="utf-8")
    print("wrote", out, "(plain svg)")


if __name__ == "__main__":
    main()
