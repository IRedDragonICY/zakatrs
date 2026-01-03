#!/bin/bash
# Script to generate type definitions and JSON schemas
# Requires: cargo install typeshare-cli

set -e

echo "=== ZakatRS Type & Schema Generation ==="
echo ""

# Create output directories
mkdir -p pkg schemas

# ============================================================
# TypeShare Generation (TypeScript & Kotlin)
# ============================================================
echo "[1/2] Generating TypeScript definitions..."
if command -v typeshare &> /dev/null; then
    typeshare . --lang=typescript --output-file=pkg/definitions.ts
    echo "      ✓ Generated: pkg/definitions.ts"
else
    echo "      ⚠ typeshare-cli not found. Install with: cargo install typeshare-cli"
fi

echo ""
echo "[1/2] Generating Kotlin definitions..."
if command -v typeshare &> /dev/null; then
    mkdir -p android/src/main/kotlin
    typeshare . --lang=kotlin --output-file=android/src/main/kotlin/ZakatTypes.kt
    echo "      ✓ Generated: android/src/main/kotlin/ZakatTypes.kt"
fi

# ============================================================
# JSON Schema Generation (via Rust example)
# ============================================================
echo ""
echo "[2/2] Generating JSON Schemas..."
cargo run --example dump_schema --quiet 2>/dev/null || {
    echo "      Building dump_schema example..."
    cargo run --example dump_schema
}

echo ""
echo "=== Generation Complete ==="
echo ""
echo "Generated files:"
ls -la pkg/*.ts 2>/dev/null || true
ls -la schemas/*.json 2>/dev/null || true
