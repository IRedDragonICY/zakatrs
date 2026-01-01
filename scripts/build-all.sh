#!/bin/bash
set -e

echo -e "\033[0;36m🚀 Starting ZakatRS Master Build...\033[0m"

# 1. Native Rust Build
echo -e "\n\033[1;33m🦀 Building Native Rust (Release)...\033[0m"
cargo build --release

# 2. Python Build (Maturin)
echo -e "\n\033[1;33m🐍 Building Python Package (Maturin)...\033[0m"
if command -v maturin &> /dev/null; then
    maturin build --release
else
    echo "⚠️ 'maturin' not in PATH, trying 'python3 -m maturin'..."
    python3 -m maturin build --release
fi

# 3. WASM & JSR Build
echo -e "\n\033[1;33m🕸️  Building WASM & JSR Package...\033[0m"
./scripts/build-wasm.sh

# 4. Dart/Flutter Prep
echo -e "\n\033[1;33m💙 Preparing Dart/Flutter Package...\033[0m"
./scripts/build-dart.sh

echo -e "\n\033[0;32m✅✅✅ ALL BUILDS COMPLETE! ✅✅✅\033[0m"
echo " - Rust: target/release"
echo " - Python: target/wheels"
echo " - WASM/JS: pkg/"
echo " - Dart: zakat_dart/"
