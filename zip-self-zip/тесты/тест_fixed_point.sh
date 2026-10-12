#!/usr/bin/env bash
# Финальный тест fixed-point самокомпиляции (zip-self-zip)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "=== ФИНАЛЬНЫЙ ТЕСТ FIXED-POINT (zip-self-zip) ==="

TMP="${TMPDIR:-/tmp}"
V1="$TMP/selfc_v1.zipb"
V2="$TMP/selfc_v2.zipb"
V3="$TMP/selfc_v3.zipb"
SRC="zip-self-zip/главное.zip"

echo "[1/5] C-bootstrap..."
if [[ -x bootstrap/zipc ]]; then
  ZIPC=bootstrap/zipc
elif [[ -x bootstrap/zipc.exe ]]; then
  ZIPC=bootstrap/zipc.exe
else
  (cd bootstrap && make clean >/dev/null 2>&1 || true; make >/dev/null 2>&1 || pwsh -File build.ps1)
  ZIPC=bootstrap/zipc
  [[ -x bootstrap/zipc.exe ]] && ZIPC=bootstrap/zipc.exe
fi
echo "✓ $ZIPC"

echo "[2/5] C-bootstrap → selfc_v1..."
"$ZIPC" "$SRC" "$V1"
echo "✓ $V1"

echo "[3/5] selfc_v1 компилирует себя..."
VM=zip-vm/target/release/zip-vm
[[ -x "${VM}.exe" ]] && VM="${VM}.exe"
if [[ ! -x "$VM" ]]; then
  cargo build --release --manifest-path zip-vm/Cargo.toml
fi
"$VM" "$V1" "$SRC" "$V2"
echo "✓ $V2"

echo "[4/5] selfc_v2 компилирует себя..."
"$VM" "$V2" "$SRC" "$V3"
echo "✓ $V3"

echo "[5/5] Сравнение хешей..."
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
echo "  v1: $H1"
echo "  v2: $H2"
echo "  v3: $H3"

if [[ "$H1" == "$H2" && "$H2" == "$H3" ]]; then
  echo "✅ FIXED-POINT ДОСТИГНУТ"
  exit 0
fi
echo "⚠️  Хеши различаются"
exit 1
