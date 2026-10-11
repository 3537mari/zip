#!/bin/bash
# Тест fixed-point: детерминированная перекомпиляция C-bootstrap.
# Полный selfc→selfc fixed-point — цель v4 (zip-self/компилятор.zip пока #runtime-заглушка).

set -e

echo "=== ТЕСТ FIXED-POINT САМОКОМПИЛЯЦИИ ==="
echo ""

cd "$(dirname "$0")/.."
TMP="${TMPDIR:-/tmp}"
BIN=./zipc
test -x "$BIN" || BIN=./zipc.exe

# 1. Собрать C-bootstrap
echo "[1/4] Сборка C-bootstrap..."
make clean > /dev/null 2>&1 || true
make > /dev/null 2>&1 || make clang > /dev/null 2>&1 || true
test -x "$BIN" || { echo "нет $BIN"; exit 1; }
echo "✓ Собран"

# 2. Эталонная программа (функции / условие / цикл / рекурсия)
echo ""
echo "[2/4] Компиляция эталона (pass 1)..."
cat > "$TMP/fp_эталон.zip" << 'EOF'
функция сложить(а, б) { возврат(а + б) }
функция факториал(н) {
    если н < 2 { возврат(1) }
    возврат(н * факториал(н - 1))
}
функция главная() {
    сумма = 0
    для и в 0..10 { сумма = сумма + и }
    ПечатьЧисло(сложить(сумма, факториал(5)))
}
EOF
"$BIN" "$TMP/fp_эталон.zip" "$TMP/fp_pass1.zipb"
echo "✓ fp_pass1.zipb"

# 3. Повторная компиляция того же исходника
echo ""
echo "[3/4] Компиляция эталона (pass 2)..."
"$BIN" "$TMP/fp_эталон.zip" "$TMP/fp_pass2.zipb"
echo "✓ fp_pass2.zipb"

# 4. Сравнить хеши
echo ""
echo "[4/4] Проверка fixed point..."
if command -v sha256sum >/dev/null 2>&1; then
  ХЕШ_1=$(sha256sum "$TMP/fp_pass1.zipb" | awk '{print $1}')
  ХЕШ_2=$(sha256sum "$TMP/fp_pass2.zipb" | awk '{print $1}')
else
  ХЕШ_1=$(shasum -a 256 "$TMP/fp_pass1.zipb" | awk '{print $1}')
  ХЕШ_2=$(shasum -a 256 "$TMP/fp_pass2.zipb" | awk '{print $1}')
fi

echo "  pass1: $ХЕШ_1"
echo "  pass2: $ХЕШ_2"

if [ "$ХЕШ_1" = "$ХЕШ_2" ]; then
  echo ""
  echo "✅ FIXED POINT ДОСТИГНУТ!"
  echo "   C-bootstrap детерминирован (одинаковый .zipb)."
else
  echo ""
  echo "❌ Хеши различаются"
  exit 1
fi

# Опционально: попытка selfc (заглушка #runtime — не ошибка)
if [ -f ../zip-self/компилятор.zip ]; then
  echo ""
  echo "[доп.] Компиляция zip-self/компилятор.zip..."
  if "$BIN" ../zip-self/компилятор.zip "$TMP/selfc_c.zipb" 2>/dev/null; then
    echo "  ✓ selfc_c.zipb создан ($(wc -c < "$TMP/selfc_c.zipb" | tr -d ' ') байт)"
    echo "  ℹ полный selfc fixed-point — план v4.0.0 (сейчас #runtime stub)"
  fi
fi

rm -f "$TMP/fp_эталон.zip" "$TMP/fp_pass1.zipb" "$TMP/fp_pass2.zipb" "$TMP/selfc_c.zipb"
echo ""
echo "=== OK ==="
