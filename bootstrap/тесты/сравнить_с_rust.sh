#!/bin/bash
# Сравнение C-bootstrap с Rust-bootstrap (размеры ZIPB)

set -e

cd "$(dirname "$0")/.."
TMP="${TMPDIR:-/tmp}"

echo "=== СРАВНЕНИЕ C-BOOTSTRAP С RUST-BOOTSTRAP ==="
echo ""

echo "[1/4] Сборка C-bootstrap..."
make clean > /dev/null 2>&1 || true
make > /dev/null 2>&1 || make clang > /dev/null 2>&1 || true
BIN=./zipc
test -x "$BIN" || BIN=./zipc.exe
test -x "$BIN" || { echo "нет C zipc"; exit 1; }

echo ""
echo "[2/4] Сборка Rust zipc..."
if [ -x ../zipc/target/release/zipc ] || [ -x ../zipc/target/release/zipc.exe ]; then
  echo "  (уже собран)"
else
  (cd ../zipc && cargo build --release --quiet)
fi
RUST=../zipc/target/release/zipc
test -x "$RUST" || RUST=../zipc/target/release/zipc.exe

echo ""
echo "[3/4] Компиляция одинакового файла..."

cat > "$TMP/тест_сравнение.zip" << 'EOF'
функция сложить(а, б) {
    возврат(а + б)
}

функция главная() {
    ПечатьЧисло(сложить(2, 3))
}
EOF

"$BIN" "$TMP/тест_сравнение.zip" "$TMP/из_c.zipb"
"$RUST" байткод "$TMP/тест_сравнение.zip" -o "$TMP/из_rust"

echo ""
echo "[4/4] Сравнение результатов..."
РАЗМЕР_C=$(stat -c%s "$TMP/из_c.zipb" 2>/dev/null || stat -f%z "$TMP/из_c.zipb")
RUST_OUT="$TMP/из_rust.zipb"
test -f "$RUST_OUT" || RUST_OUT="$TMP/из_rust"
РАЗМЕР_RUST=$(stat -c%s "$RUST_OUT" 2>/dev/null || stat -f%z "$RUST_OUT")

echo "  C-bootstrap:    $РАЗМЕР_C байт"
echo "  Rust-bootstrap: $РАЗМЕР_RUST байт"

if [ "$РАЗМЕР_C" = "$РАЗМЕР_RUST" ]; then
  echo ""
  echo "✅ Размеры совпадают!"
else
  echo ""
  echo "⚠️  Размеры различаются (разный уровень оптимизации / layout)"
fi

# Проверка исполнения C-артефакта
if [ -x ../zip-vm/target/release/zip-vm ] || [ -x ../zip-vm/target/release/zip-vm.exe ]; then
  VM=../zip-vm/target/release/zip-vm
  test -x "$VM" || VM=../zip-vm/target/release/zip-vm.exe
  OUT=$("$VM" "$TMP/из_c.zipb" 2>/dev/null | grep -E '^[0-9]+$' | head -1)
  echo "  C → VM вывод: $OUT (ожидание 5)"
fi

rm -f "$TMP/тест_сравнение.zip" "$TMP/из_c.zipb" "$TMP/из_rust.zipb" "$TMP/из_rust"
echo ""
echo "=== OK ==="
