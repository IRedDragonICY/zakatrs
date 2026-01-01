# Master Build Script for ZakatRS
# Builds Rust, Python, WASM, and Dart packages.

$ErrorActionPreference = "Stop"

Write-Host "🚀 Starting ZakatRS Master Build..." -ForegroundColor Cyan

# 0. Sync Versions
Write-Host "`n🔄 Synchronizing Versions..." -ForegroundColor Yellow
.\scripts\sync-versions.ps1

# 1. Native Rust Build
Write-Host "`n🦀 Building Native Rust (Release)..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "Rust build failed!" }

# 2. Python Build (Maturin)
Write-Host "`n🐍 Building Python Package (Maturin)..." -ForegroundColor Yellow
# Check if maturin is in PATH, otherwise use python -m maturin
if (Get-Command maturin -ErrorAction SilentlyContinue) {
    maturin build --release
}
else {
    Write-Host "⚠️ 'maturin' not in PATH, trying 'python -m maturin'..." -ForegroundColor Yellow
    python -m maturin build --release
}
if ($LASTEXITCODE -ne 0) { throw "Maturin build failed!" }

# 3. WASM & JSR Build
Write-Host "`n🕸️  Building WASM & JSR Package..." -ForegroundColor Yellow
if (Get-Command wasm-pack -ErrorAction SilentlyContinue) {
    .\scripts\build-wasm.ps1
    if ($LASTEXITCODE -ne 0) { throw "WASM build failed!" }
}
else {
    Write-Host "⚠️ 'wasm-pack' not found! Skipping WASM build." -ForegroundColor Red
}

# 4. Dart/Flutter Prep
Write-Host "`n💙 Preparing Dart/Flutter Package..." -ForegroundColor Yellow
.\scripts\build-dart.ps1
if ($LASTEXITCODE -ne 0) { throw "Dart build failed!" }

Write-Host "`n✅✅✅ ALL BUILDS COMPLETE! ✅✅✅" -ForegroundColor Green
Write-Host " - Rust: target/release"
Write-Host " - Python: target/wheels"
Write-Host " - WASM/JS: pkg/"
Write-Host " - Dart: zakat_dart/"
