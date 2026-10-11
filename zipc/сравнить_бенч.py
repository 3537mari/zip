# -*- coding: utf-8 -*-
"""Сравнение базовый_результат.txt vs оптимизированный_результат.txt"""
import re
from pathlib import Path

def to_ms(val: float, unit: str) -> float:
    u = unit.replace("µ", "u").replace("μ", "u").replace("┬╡", "u")
    if u == "ms":
        return val
    if u == "s":
        return val * 1000.0
    if "u" in u or u == "us":
        return val / 1000.0
    if u == "ns":
        return val / 1_000_000.0
    return val / 1000.0 if val < 100 else val

def parse(path: Path):
    text = path.read_text(encoding="utf-8", errors="replace")
    out = {}
    for line in text.splitlines():
        m = re.search(
            r"/(\d+)\s+time:\s+\[\s*([0-9.]+)\s+(\S+)\s+([0-9.]+)\s+(\S+)\s+([0-9.]+)\s+(\S+)\s*\]",
            line,
        )
        if not m:
            continue
        size = m.group(1)
        mid = float(m.group(4))
        unit = m.group(5)
        pre = line[: line.find("/" + size)].strip()
        stage = pre.split()[-1] if pre.split() else pre
        for known in ("лексика", "парсер", "ir", "байткод_v3"):
            if known in stage or stage.endswith(known):
                stage = known
                break
        # recover mojibake by order heuristics
        if stage not in ("лексика", "парсер", "ir", "байткод_v3"):
            if "ir" in line[:20]:
                stage = "ir"
            elif "time" in line:
                # fall back: count previous stages
                pass
        out[f"{stage}/{size}"] = to_ms(mid, unit)
    # Fix mojibake keys by sequential remap if needed
    fixed = {}
    order = ["лексика", "парсер", "ir", "байткод_v3"]
    by_size = {}
    for k, v in out.items():
        size = k.split("/")[-1]
        by_size.setdefault(size, []).append((k, v))
    for size, items in by_size.items():
        items_sorted = sorted(items, key=lambda x: list(out.keys()).index(x[0]))
        # if names garbled, assign by appearance order
        stages_seen = []
        for (k, v) in items_sorted:
            st = k.rsplit("/", 1)[0]
            if st in order:
                fixed[k] = v
                stages_seen.append(st)
            else:
                for cand in order:
                    nk = f"{cand}/{size}"
                    if nk not in fixed:
                        fixed[nk] = v
                        break
    return fixed if fixed else out

base = parse(Path("базовый_результат.txt"))
opt = parse(Path("оптимизированный_результат.txt"))

print(f"{'Тест':<28} {'До':>12} {'После':>12} {'Ускорение':>10}")
print("-" * 66)
speeds = []
for k in sorted(base.keys(), key=lambda x: (x.split('/')[0], int(x.split('/')[1]))):
    if k not in opt:
        continue
    sp = base[k] / opt[k] if opt[k] else float("inf")
    speeds.append(sp)
    print(f"{k:<28} {base[k]:>8.3f} ms {opt[k]:>8.3f} ms {sp:>8.2f}×")

if speeds:
    # pipeline 1000
    def pipe(d, size):
        keys = [f"лексика/{size}", f"парсер/{size}", f"ir/{size}", f"байткод_v3/{size}"]
        if all(k in d for k in keys):
            return sum(d[k] for k in keys)
        return None
    for size in ("1000", "5000"):
        b, a = pipe(base, size), pipe(opt, size)
        if b and a:
            print("-" * 66)
            print(f"{'пайплайн/'+size:<28} {b:>8.3f} ms {a:>8.3f} ms {b/a:>8.2f}×")
