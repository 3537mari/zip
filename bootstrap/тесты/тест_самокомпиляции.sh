#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "=== ТЕСТ BOOTSTRAP ZIPB ==="
make clean >/dev/null 2>&1 || true
make >/dev/null 2>&1 || make clang >/dev/null 2>&1
BIN=./zipc
test -x "$BIN" || BIN=./zipc.exe
printf '%s\n' 'ПечатьЧисло(42)' > /tmp/тест.zip
"$BIN" /tmp/тест.zip /tmp/тест.zipb
MAGIC=$(od -An -tx1 -N4 /tmp/тест.zipb 2>/dev/null | tr -d ' \n' || xxd -p -l 4 /tmp/тест.zipb)
echo "magic hex: $MAGIC"
echo "$MAGIC" | grep -qi '5a495042'
VER=$(od -An -tx1 -j4 -N2 /tmp/тест.zipb 2>/dev/null | tr -d ' \n' || xxd -p -s 4 -l 2 /tmp/тест.zipb)
echo "version hex: $VER"
echo "$VER" | grep -qi '0300'
if [ -f ../zip-self/компилятор.zip ]; then
  "$BIN" ../zip-self/компилятор.zip /tmp/компилятор_v1.zipb || true
  echo "selfc compile attempted"
fi
echo "=== OK ==="
rm -f /tmp/тест.zip /tmp/тест.zipb
