#!/bin/bash
set -e
cd "$(dirname "$0")/.."
make >/dev/null
printf '%s\n' 'ПечатьСтроки("привет")' > /tmp/p.zip
./zipc /tmp/p.zip /tmp/p.zipb
echo "парсер OK"
