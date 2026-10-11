#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
if command -v pwsh >/dev/null 2>&1; then
  pwsh -File "$DIR/запустить_все.ps1"
else
  powershell -File "$DIR/запустить_все.ps1"
fi
