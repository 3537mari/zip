#!/usr/bin/env bash
# Сравнивает вывод Rust-лексера и ZIP-лексера на одном исходнике
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ИСХОДНИК="${1:-$ROOT/examples/vm_demo.zip}"

echo "=== Rust-лексер ==="
"$ROOT/zipc/target/release/zipc" "$ИСХОДНИК" --только-лексика || \
  "$ROOT/zipc/target/debug/zipc" "$ИСХОДНИК" --только-лексика

echo ""
echo "=== ZIP-лексер ==="
ZIPC="$ROOT/zipc/target/release/zipc"
[[ -x "$ZIPC" ]] || ZIPC="$ROOT/zipc/target/debug/zipc"
"$ZIPC" "$ROOT/zip-self/лексика.zip" -o "$ROOT/zip-self/лексер_zip" --llvm
"$ROOT/zip-self/лексер_zip" "$ИСХОДНИК"
