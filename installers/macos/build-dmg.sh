#!/bin/bash
set -e
VERSION="3.0.0"
BUILD_DIR="$(pwd)/build-dmg"
APP_NAME="ZIP"
ROOT="$(cd ../.. && pwd)"
echo "=== DMG ZIP ==="
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$APP_NAME.app/Contents/MacOS"
cd "$ROOT"
cargo build --release --manifest-path zipc/Cargo.toml
cargo build --release --manifest-path zip-vm/Cargo.toml
cd installers/macos
cp "$ROOT/zipc/target/release/zipc" "$BUILD_DIR/$APP_NAME.app/Contents/MacOS/"
cp "$ROOT/zip-vm/target/release/zip-vm" "$BUILD_DIR/$APP_NAME.app/Contents/MacOS/"
cat > "$BUILD_DIR/$APP_NAME.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleIdentifier</key><string>org.zip-lang.zip</string>
  <key>CFBundleExecutable</key><string>zipc</string>
</dict></plist>
EOF
hdiutil create -volname "$APP_NAME" -srcfolder "$BUILD_DIR" -ov -format UDZO "ZIP-${VERSION}-macos.dmg"
echo "OK: ZIP-${VERSION}-macos.dmg"
