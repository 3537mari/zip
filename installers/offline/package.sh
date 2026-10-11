#!/bin/bash
set -e
VERSION="3.0.0"
OUTPUT="ZIP-${VERSION}-offline-installer.sh"
ROOT="$(cd ../.. && pwd)"
TEMP_DIR="$(mktemp -d)"
trap "rm -rf $TEMP_DIR" EXIT
mkdir -p "$TEMP_DIR/bootstrap"
cp -r "$ROOT/bootstrap/"* "$TEMP_DIR/bootstrap/"
cp -r "$ROOT/zip-self" "$TEMP_DIR/" 2>/dev/null || true
cp "$ROOT/README.md" "$ROOT/LICENSE" "$TEMP_DIR/" 2>/dev/null || true
cp install.sh "$TEMP_DIR/"
cd "$TEMP_DIR"
tar czf /tmp/zip-payload.tar.gz .
{
  cat << 'WRAPPER_EOF'
#!/bin/bash
set -e
TEMP_DIR="$(mktemp -d)"
trap "rm -rf $TEMP_DIR" EXIT
PAYLOAD_LINE=$(awk '/^__PAYLOAD_BELOW__/ {print NR + 1; exit 0; }' "$0")
tail -n +$PAYLOAD_LINE "$0" | tar xz -C "$TEMP_DIR"
cd "$TEMP_DIR"
bash install.sh
exit 0
__PAYLOAD_BELOW__
WRAPPER_EOF
  cat /tmp/zip-payload.tar.gz
} > "$ROOT/installers/offline/$OUTPUT"
chmod +x "$ROOT/installers/offline/$OUTPUT"
echo "OK: $OUTPUT"
