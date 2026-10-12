#!/usr/bin/env bash
# Финальный тест: fixed-point stage-1+ без C (VM-генератор)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "=== ФИНАЛЬНЫЙ ТЕСТ FIXED-POINT БЕЗ C ==="
echo ""

TMP="${TMPDIR:-/tmp}"
V1="$TMP/selfc_v1.zipb"
V2="$TMP/selfc_v2.zipb"
V3="$TMP/selfc_v3.zipb"
SRC="zip-self-zip/главное_чистое.zip"

echo "[1/5] Сборка C-bootstrap (stage-0, один раз)..."
if [[ -x bootstrap/zipc ]]; then
  ZIPC=bootstrap/zipc
elif [[ -x bootstrap/zipc.exe ]]; then
  ZIPC=bootstrap/zipc.exe
else
  (cd bootstrap && make clean >/dev/null 2>&1 || true
   make >/dev/null 2>&1 || pwsh -File build.ps1)
  ZIPC=bootstrap/zipc
  [[ -x bootstrap/zipc.exe ]] && ZIPC=bootstrap/zipc.exe
fi
echo "✓ $ZIPC"

echo ""
echo "[2/5] C-bootstrap → selfc_v1 (драйвер)..."
"$ZIPC" "$SRC" "$V1"
echo "✓ $V1"

echo ""
echo "[3/5] selfc_v1 компилирует себя через VM (без C)..."
cargo build --release --manifest-path zip-vm/Cargo.toml
VM=""
if [[ -n "${CARGO_TARGET_DIR:-}" && -x "$CARGO_TARGET_DIR/release/zip-vm" ]]; then
  VM="$CARGO_TARGET_DIR/release/zip-vm"
elif [[ -n "${CARGO_TARGET_DIR:-}" && -x "$CARGO_TARGET_DIR/release/zip-vm.exe" ]]; then
  VM="$CARGO_TARGET_DIR/release/zip-vm.exe"
elif [[ -x zip-vm/target/release/zip-vm ]]; then
  VM=zip-vm/target/release/zip-vm
elif [[ -x zip-vm/target/release/zip-vm.exe ]]; then
  VM=zip-vm/target/release/zip-vm.exe
fi
[[ -n "$VM" ]] || { echo "zip-vm не найден"; exit 1; }
echo "VM: $VM"
unset ZIP_USE_C_BOOTSTRAP || true
"$VM" "$V1" "$SRC" "$V2"
echo "✓ $V2"

echo ""
echo "[4/5] selfc_v2 компилирует себя через VM..."
"$VM" "$V2" "$SRC" "$V3"
echo "✓ $V3"

echo ""
echo "[5/5] Проверка fixed-point (v2 == v3; stage-1+ без C)..."
hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    python -c "import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest())" "$1"
  fi
}
H1=$(hash_file "$V1")
H2=$(hash_file "$V2")
H3=$(hash_file "$V3")
echo "  v1 (C stage-0): $H1"
echo "  v2 (VM):        $H2"
echo "  v3 (VM):        $H3"

if [[ "$H2" == "$H3" ]]; then
  echo ""
  echo "✅ FIXED-POINT ДОСТИГНУТ БЕЗ C (v2 == v3)"
  if [[ "$H1" != "$H2" ]]; then
    echo "ℹ️  v1≠v2 ожидаемо: stage-0 C и stage-1 VM-генератор различаются"
  fi
  echo "ZIP stage-1+ самодостаточен (только VM)."
  exit 0
fi
echo ""
echo "⚠️  Хеши v2/v3 различаются"
exit 1
