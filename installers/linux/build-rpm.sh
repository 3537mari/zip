#!/bin/bash
set -e
ROOT="$(cd ../.. && pwd)"
echo "=== rpmbuild ZIP ==="
if ! command -v rpmbuild >/dev/null; then
  echo "rpmbuild not found"; exit 1
fi
rpmbuild -bb zip.spec --define "_topdir $(pwd)/rpmbuild" --define "_sourcedir $ROOT" || true
echo "See rpmbuild/RPMS/"
