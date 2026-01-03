# ============================================================
# ZakatRS - Multi-Platform Binding Generator (Windows)
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

$ErrorActionPreference = "Stop"
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$ROOT_DIR = Split-Path -Parent $SCRIPT_DIR
Set-Location $ROOT_DIR

Write-Host "╔══════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     ZakatRS - Multi-Platform Binding Generator           ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Create output directories
New-Item -ItemType Directory -Force -Path pkg, schemas, "android/src/main/kotlin" | Out-Null

# ============================================================
# 1. TypeShare: Generate TypeScript & Kotlin Types
# ============================================================
Write-Host "┌─────────────────────────────────────────────────────────┐" -ForegroundColor Yellow
Write-Host "│ [1/5] TypeShare: TypeScript & Kotlin Types              │" -ForegroundColor Yellow
Write-Host "└─────────────────────────────────────────────────────────┘" -ForegroundColor Yellow

$typeshare = Get-Command typeshare -ErrorAction SilentlyContinue
if ($typeshare) {
    Write-Host "  → Generating TypeScript definitions..." -ForegroundColor Gray
    typeshare . --lang=typescript --output-file=pkg/zakat.types.ts
    Write-Host "    ✓ pkg/zakat.types.ts" -ForegroundColor Green
    
    Write-Host "  → Generating Kotlin definitions..." -ForegroundColor Gray
    typeshare . --lang=kotlin --output-file=android/src/main/kotlin/ZakatTypes.kt
    Write-Host "    ✓ android/src/main/kotlin/ZakatTypes.kt" -ForegroundColor Green
}
else {
    Write-Host "  ⚠ typeshare-cli not found. Install with:" -ForegroundColor Yellow
    Write-Host "    cargo install typeshare-cli" -ForegroundColor Gray
}

# ============================================================
# 2. JSON Schema Generation (via Rust example)
# ============================================================
Write-Host ""
Write-Host "┌─────────────────────────────────────────────────────────┐" -ForegroundColor Yellow
Write-Host "│ [2/5] JSON Schema Generation                            │" -ForegroundColor Yellow
Write-Host "└─────────────────────────────────────────────────────────┘" -ForegroundColor Yellow

Write-Host "  → Running schema generator..." -ForegroundColor Gray
cargo run --example dump_schema
Write-Host "    ✓ schemas/*.json" -ForegroundColor Green

# ============================================================
# 3. WASM Package Build
# ============================================================
Write-Host ""
Write-Host "┌─────────────────────────────────────────────────────────┐" -ForegroundColor Yellow
Write-Host "│ [3/5] WASM Package Build                                │" -ForegroundColor Yellow
Write-Host "└─────────────────────────────────────────────────────────┘" -ForegroundColor Yellow

$wasmpack = Get-Command wasm-pack -ErrorAction SilentlyContinue
if ($wasmpack) {
    Write-Host "  → Building WASM package..." -ForegroundColor Gray
    wasm-pack build --target web --out-dir pkg/wasm --features wasm
    Write-Host "    ✓ pkg/wasm/" -ForegroundColor Green
    
    # Copy TypeScript types to WASM package
    if (Test-Path "pkg/zakat.types.ts") {
        Copy-Item "pkg/zakat.types.ts" "pkg/wasm/"
        Write-Host "    ✓ Copied TypeScript types to pkg/wasm/" -ForegroundColor Green
    }
}
else {
    Write-Host "  ⚠ wasm-pack not found. Install with:" -ForegroundColor Yellow
    Write-Host "    cargo install wasm-pack" -ForegroundColor Gray
}

# ============================================================
# 4. UniFFI Bindings (Kotlin Native)
# ============================================================
Write-Host ""
Write-Host "┌─────────────────────────────────────────────────────────┐" -ForegroundColor Yellow
Write-Host "│ [4/5] UniFFI Kotlin Bindings                            │" -ForegroundColor Yellow
Write-Host "└─────────────────────────────────────────────────────────┘" -ForegroundColor Yellow

if (Test-Path "src/zakat.udl") {
    $uniffi = Get-Command uniffi-bindgen -ErrorAction SilentlyContinue
    if ($uniffi) {
        Write-Host "  → Generating Kotlin native bindings..." -ForegroundColor Gray
        cargo build --release --features uniffi
        uniffi-bindgen generate src/zakat.udl --language kotlin --out-dir android/src/main/kotlin
        Write-Host "    ✓ android/src/main/kotlin/" -ForegroundColor Green
    }
    else {
        Write-Host "  ⚠ uniffi-bindgen not found. Install with:" -ForegroundColor Yellow
        Write-Host "    cargo install uniffi-bindgen-cli" -ForegroundColor Gray
    }
}
else {
    Write-Host "  ⓘ UniFFI not configured (no src/zakat.udl found)" -ForegroundColor Cyan
    Write-Host "    TypeShare-generated types are available instead." -ForegroundColor Gray
}

# ============================================================
# 5. Dart/Flutter Bindings
# ============================================================
Write-Host ""
Write-Host "┌─────────────────────────────────────────────────────────┐" -ForegroundColor Yellow
Write-Host "│ [5/5] Dart/Flutter Bindings                             │" -ForegroundColor Yellow
Write-Host "└─────────────────────────────────────────────────────────┘" -ForegroundColor Yellow

$DART_DIR = Join-Path $ROOT_DIR "zakat_dart"
if (Test-Path $DART_DIR) {
    $frb = Get-Command flutter_rust_bridge_codegen -ErrorAction SilentlyContinue
    if ($frb) {
        Write-Host "  → Generating Dart bindings..." -ForegroundColor Gray
        Push-Location $DART_DIR
        flutter_rust_bridge_codegen generate
        Pop-Location
        Write-Host "    ✓ Dart bindings generated" -ForegroundColor Green
    }
    else {
        Write-Host "  ⚠ flutter_rust_bridge_codegen not found. Install with:" -ForegroundColor Yellow
        Write-Host "    cargo install flutter_rust_bridge_codegen" -ForegroundColor Gray
    }
}
else {
    Write-Host "  ⓘ zakat_dart directory not found, skipping Dart bindings" -ForegroundColor Cyan
}

# ============================================================
# Summary
# ============================================================
Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    Generation Complete                    ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Generated artifacts:" -ForegroundColor White

if (Test-Path "pkg/zakat.types.ts") {
    Write-Host "  TypeScript:  pkg/zakat.types.ts" -ForegroundColor Green
}

if (Test-Path "android/src/main/kotlin/ZakatTypes.kt") {
    Write-Host "  Kotlin:      android/src/main/kotlin/ZakatTypes.kt" -ForegroundColor Green
}

if (Test-Path "schemas") {
    Write-Host "  JSON Schemas:" -ForegroundColor Green
    Get-ChildItem schemas/*.json | ForEach-Object {
        Write-Host "                - $($_.Name)" -ForegroundColor Gray
    }
}

if (Test-Path "pkg/wasm") {
    Write-Host "  WASM:        pkg/wasm/" -ForegroundColor Green
}

$dartRust = Join-Path $DART_DIR "lib/src/rust"
if (Test-Path $dartRust) {
    Write-Host "  Dart:        zakat_dart/lib/src/rust/" -ForegroundColor Green
}

Write-Host ""
Write-Host "Done! 🎉" -ForegroundColor Green
