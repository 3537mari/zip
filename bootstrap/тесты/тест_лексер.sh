#!/bin/bash
set -e
cd "$(dirname "$0")/.."
make clean >/dev/null 2>&1 || true
make >/dev/null
printf '%s\n' 'функция главная() { возврат(1) }' > /tmp/lex.zip
./zipc /tmp/lex.zip /tmp/lex.zipb
test -f /tmp/lex.zipb
echo "лексер/парсер OK"
