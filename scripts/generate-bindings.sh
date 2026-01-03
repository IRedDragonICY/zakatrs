#!/bin/bash
# ============================================================
# ZakatRS - Multi-Platform Binding Generator
# ============================================================
# This script generates bindings for all supported platforms:
#   - TypeScript types (via typeshare)
#   - Kotlin types (via typeshare)
#   - JSON Schemas (via Rust example)
#   - WASM package (via wasm-pack)
#   - Dart bindings (via flutter_rust_bridge)
#
# Prerequisites:
#   cargo install typeshare-cli
#   cargo install wasm-pack
#   cargo install uniffi-bindgen-cli (for UniFFI)
#   flutter_rust_bridge_codegen (for Dart)
# ============================================================

set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║     ZakatRS - Multi-Platform Binding Generator           ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Create output directories
mkdir -p pkg schemas android/src/main/kotlin

# ============================================================
# 1. TypeShare: Generate TypeScript & Kotlin Types
# ============================================================
echo "┌─────────────────────────────────────────────────────────┐"
echo "│ [1/5] TypeShare: TypeScript & Kotlin Types              │"
echo "└─────────────────────────────────────────────────────────┘"

if command -v typeshare &> /dev/null; then
    echo "  → Generating TypeScript definitions..."
    typeshare . --lang=typescript --output-file=pkg/zakat.types.ts
    echo "    ✓ pkg/zakat.types.ts"
    
    echo "  → Generating Kotlin definitions..."
    typeshare . --lang=kotlin --output-file=android/src/main/kotlin/ZakatTypes.kt
    echo "    ✓ android/src/main/kotlin/ZakatTypes.kt"
else
    echo "  ⚠ typeshare-cli not found. Install with:"
    echo "    cargo install typeshare-cli"
fi

# ============================================================
# 2. JSON Schema Generation (via Rust example)
# ============================================================
echo ""
echo "┌─────────────────────────────────────────────────────────┐"
echo "│ [2/5] JSON Schema Generation                            │"
echo "└─────────────────────────────────────────────────────────┘"

echo "  → Running schema generator..."
cargo run --example dump_schema --quiet 2>/dev/null || cargo run --example dump_schema
echo "    ✓ schemas/*.json"

# ============================================================
# 3. WASM Package Build
# ============================================================
echo ""
echo "┌─────────────────────────────────────────────────────────┐"
echo "│ [3/5] WASM Package Build                                │"
echo "└─────────────────────────────────────────────────────────┘"

if command -v wasm-pack &> /dev/null; then
    echo "  → Building WASM package..."
    wasm-pack build --target web --out-dir pkg/wasm --features wasm
    echo "    ✓ pkg/wasm/"
    
    # Copy TypeScript types to WASM package
    if [ -f "pkg/zakat.types.ts" ]; then
        cp pkg/zakat.types.ts pkg/wasm/
        echo "    ✓ Copied TypeScript types to pkg/wasm/"
    fi
else
    echo "  ⚠ wasm-pack not found. Install with:"
    echo "    cargo install wasm-pack"
fi

# ============================================================
# 4. UniFFI Bindings (Kotlin Native)
# ============================================================
echo ""
echo "┌─────────────────────────────────────────────────────────┐"
echo "│ [4/5] UniFFI Kotlin Bindings                            │"
echo "└─────────────────────────────────────────────────────────┘"

# Check if uniffi feature exists and UDL file is present
if [ -f "src/zakat.udl" ] && cargo metadata --format-version=1 2>/dev/null | grep -q '"uniffi"'; then
    if command -v uniffi-bindgen &> /dev/null; then
        echo "  → Generating Kotlin native bindings..."
        cargo build --release --features uniffi
        uniffi-bindgen generate src/zakat.udl --language kotlin --out-dir android/src/main/kotlin
        echo "    ✓ android/src/main/kotlin/"
    else
        echo "  ⚠ uniffi-bindgen not found. Install with:"
        echo "    cargo install uniffi-bindgen-cli"
    fi
else
    echo "  ⓘ UniFFI not configured (no src/zakat.udl found)"
    echo "    TypeShare-generated types are available instead."
fi

# ============================================================
# 5. Dart/Flutter Bindings
# ============================================================
echo ""
echo "┌─────────────────────────────────────────────────────────┐"
echo "│ [5/5] Dart/Flutter Bindings                             │"
echo "└─────────────────────────────────────────────────────────┘"

DART_DIR="$ROOT_DIR/zakat_dart"
if [ -d "$DART_DIR" ]; then
    if command -v flutter_rust_bridge_codegen &> /dev/null; then
        echo "  → Generating Dart bindings..."
        cd "$DART_DIR"
        flutter_rust_bridge_codegen generate
        echo "    ✓ Dart bindings generated"
        cd "$ROOT_DIR"
    else
        echo "  ⚠ flutter_rust_bridge_codegen not found. Install with:"
        echo "    cargo install flutter_rust_bridge_codegen"
        echo "    Or: dart pub global activate flutter_rust_bridge"
    fi
else
    echo "  ⓘ zakat_dart directory not found, skipping Dart bindings"
fi

# ============================================================
# Summary
# ============================================================
echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║                    Generation Complete                    ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Generated artifacts:"
echo ""

if [ -f "pkg/zakat.types.ts" ]; then
    echo "  TypeScript:  pkg/zakat.types.ts"
fi

if [ -f "android/src/main/kotlin/ZakatTypes.kt" ]; then
    echo "  Kotlin:      android/src/main/kotlin/ZakatTypes.kt"
fi

if [ -d "schemas" ]; then
    echo "  JSON Schemas: schemas/*.json"
    ls schemas/*.json 2>/dev/null | while read f; do
        echo "                - $(basename $f)"
    done
fi

if [ -d "pkg/wasm" ]; then
    echo "  WASM:        pkg/wasm/"
fi

if [ -d "$DART_DIR/lib/src/rust" ]; then
    echo "  Dart:        zakat_dart/lib/src/rust/"
fi

echo ""
echo "Done! 🎉"
