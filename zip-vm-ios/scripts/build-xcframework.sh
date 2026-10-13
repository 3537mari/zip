#!/usr/bin/env bash
# Сборка ZipVM.xcframework для устройства + симулятора.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

echo "==> cargo build (device arm64)"
cargo build --release --target aarch64-apple-ios

echo "==> cargo build (sim arm64)"
cargo build --release --target aarch64-apple-ios-sim

echo "==> cargo build (sim x86_64)"
cargo build --release --target x86_64-apple-ios

mkdir -p target/ios-sim-universal/release
echo "==> lipo universal simulator"
lipo -create \
  target/aarch64-apple-ios-sim/release/libzip_vm_ios.a \
  target/x86_64-apple-ios/release/libzip_vm_ios.a \
  -output target/ios-sim-universal/release/libzip_vm_ios.a

lipo -info target/ios-sim-universal/release/libzip_vm_ios.a

rm -rf ZipVM.xcframework
echo "==> xcodebuild -create-xcframework"
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libzip_vm_ios.a \
    -headers include \
  -library target/ios-sim-universal/release/libzip_vm_ios.a \
    -headers include \
  -output ZipVM.xcframework

echo "==> done"
ls -la ZipVM.xcframework/
find ZipVM.xcframework -type f | sort
