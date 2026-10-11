#!/bin/bash
# Проверка документации ZIP на консистентность

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== Проверка документации ZIP ==="
echo ""

# 1. Все ли файлы на месте?
echo "1. Проверка обязательных файлов:"
ОБЯЗАТЕЛЬНЫЕ=(
    "README.md"
    "CHANGELOG.md"
    "ROADMAP.md"
    "МАСТЕР-ПЛАН.md"
    "docs/УЧЕБНИК.md"
    "docs/СПРАВОЧНИК.md"
    "docs/STD.md"
    "docs/COOKBOOK.md"
    "docs/FAQ.md"
    "docs/МИГРАЦИЯ.md"
    "docs/ИНДЕКС.md"
)

for файл in "${ОБЯЗАТЕЛЬНЫЕ[@]}"; do
    if [ -f "$файл" ]; then
        echo "  ✓ $файл"
    else
        echo "  ❌ $файл — ОТСУТСТВУЕТ"
    fi
done

echo ""

# 2. Проверка битых ссылок (относительные .md)
echo "2. Проверка ссылок:"
БИТЫХ=0
python3 - "$ROOT" <<'PY' || python - "$ROOT" <<'PY'
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
broken = 0
for path in sorted((root / "docs").glob("*.md")):
    text = path.read_text(encoding="utf-8")
    for m in re.finditer(r"\]\(([^)]+\.md)(#[^)]*)?\)", text):
        rel = m.group(1)
        if rel.startswith("http"):
            continue
        candidates = [
            (path.parent / rel).resolve(),
            (root / rel).resolve(),
            (root / "docs" / rel).resolve(),
        ]
        if not any(c.exists() for c in candidates):
            print(f"  ❌ {path.relative_to(root)} → {rel}")
            broken += 1
for name in ["README.md", "CHANGELOG.md", "ROADMAP.md", "МАСТЕР-ПЛАН.md"]:
    path = root / name
    if not path.exists():
        continue
    text = path.read_text(encoding="utf-8")
    for m in re.finditer(r"\]\(([^)]+\.md)(#[^)]*)?\)", text):
        rel = m.group(1)
        if rel.startswith("http"):
            continue
        candidates = [
            (path.parent / rel).resolve(),
            (root / rel).resolve(),
        ]
        if not any(c.exists() for c in candidates):
            print(f"  ❌ {name} → {rel}")
            broken += 1
if broken == 0:
    print("  ✓ Битых ссылок нет")
sys.exit(0 if broken == 0 else 1)
PY
БИТЫХ=$?

echo ""

# 3. Размер документации
echo "3. Размер документации:"
du -sh docs/ 2>/dev/null || python -c "from pathlib import Path; b=sum(p.stat().st_size for p in Path('docs').rglob('*') if p.is_file()); print(f'{b/1024:.0f}K\tdocs/')"
echo ""

# 4. Количество строк
echo "4. Строк в документации:"
wc -l docs/*.md 2>/dev/null | tail -1 || python -c "from pathlib import Path; print(sum(len(p.read_text(encoding='utf-8').splitlines()) for p in Path('docs').glob('*.md')), 'total')"
echo ""

# 5. Проверка UTF-8
echo "5. Проверка UTF-8:"
python - <<'PY'
from pathlib import Path
ok = True
for p in list(Path('docs').glob('*.md')) + [Path('README.md'), Path('CHANGELOG.md'), Path('ROADMAP.md'), Path('МАСТЕР-ПЛАН.md')]:
    try:
        p.read_text(encoding='utf-8')
    except Exception as e:
        print(f'  ❌ {p} — {e}')
        ok = False
if ok:
    print('  ✓ Все файлы в UTF-8')
PY

echo ""
echo "=== Проверка завершена ==="
exit $БИТЫХ
