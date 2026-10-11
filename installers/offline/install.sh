#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ZIP_VERSION="3.0.0"
echo "=== Offline install ZIP v${ZIP_VERSION} ==="
if ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; then
  echo "Need gcc or clang"; exit 1
fi
cd "$SCRIPT_DIR/../../bootstrap" 2>/dev/null || cd "$SCRIPT_DIR/bootstrap"
make clean >/dev/null 2>&1 || true
make >/dev/null 2>&1 || clang -O3 -o zipc утилиты.c лексер.c парсер.c генератор.c главное.c
INSTALL_DIR="${HOME}/.local/zip"
mkdir -p "$INSTALL_DIR/bin"
cp zipc "$INSTALL_DIR/bin/" 2>/dev/null || cp zipc.exe "$INSTALL_DIR/bin/"
SHELL_RC="$HOME/.bashrc"
[ -n "$ZSH_VERSION" ] && SHELL_RC="$HOME/.zshrc"
if ! grep -q "$INSTALL_DIR/bin" "$SHELL_RC" 2>/dev/null; then
  echo "export PATH=\"$INSTALL_DIR/bin:\$PATH\"" >> "$SHELL_RC"
fi
echo "Installed to $INSTALL_DIR"
