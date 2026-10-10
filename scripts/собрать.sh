#!/bin/bash
set -e
cd "$(dirname "$0")/.."
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
cargo build --release --manifest-path zipc/Cargo.toml
echo "✓ Компилятор ZIP собран"
echo "Путь: $(pwd)/zipc/target/release/zipc"