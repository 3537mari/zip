#!/bin/bash
set -e
VERSION="3.0.0"
ARCH="amd64"
PKG_NAME="zip-lang"
BUILD_DIR="$(pwd)/build-deb"
ROOT="$(cd ../.. && pwd)"
echo "=== .deb ZIP v${VERSION} ==="
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/DEBIAN" "$BUILD_DIR/usr/local/bin" "$BUILD_DIR/usr/share/doc/zip-lang"
cd "$ROOT"
cargo build --release --manifest-path zipc/Cargo.toml
cargo build --release --manifest-path zip-vm/Cargo.toml
cd installers/linux
cp "$ROOT/zipc/target/release/zipc" "$BUILD_DIR/usr/local/bin/"
cp "$ROOT/zip-vm/target/release/zip-vm" "$BUILD_DIR/usr/local/bin/"
cp "$ROOT/README.md" "$ROOT/CHANGELOG.md" "$BUILD_DIR/usr/share/doc/zip-lang/" 2>/dev/null || true
cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: $PKG_NAME
Version: $VERSION
Architecture: $ARCH
Maintainer: ZIP Community <zip-lang@proton.me>
Depends: libc6 (>= 2.31)
Section: devel
Priority: optional
Homepage: https://github.com/3537mari/zip
Description: ZIP — Cyrillic programming language
 Compiler and VM.
EOF
dpkg-deb --build "$BUILD_DIR" "${PKG_NAME}_${VERSION}_${ARCH}.deb"
echo "OK: ${PKG_NAME}_${VERSION}_${ARCH}.deb"
