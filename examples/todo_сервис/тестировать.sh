#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
if command -v pwsh >/dev/null 2>&1; then
  pwsh -File "$DIR/тестировать.ps1"
else
  powershell -File "$DIR/тестировать.ps1"
fi
