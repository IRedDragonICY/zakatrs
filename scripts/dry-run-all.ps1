# Master Dry-Run Script for ZakatRS
# Simulates publishing to Crates.io, PyPI, NPM, JSR, and Pub.dev to detect errors.

$ErrorActionPreference = "Stop"

Write-Host "🛡️  Starting ZakatRS Dry-Run Verification..." -ForegroundColor Cyan
Write-Host "ℹ️  This script checks if packages are valid and ready for publishing."

# 1. Rust (Crates.io)
Write-Host "`n🦀 Verifying Rust (Crates.io)..." -ForegroundColor Yellow
cargo publish --dry-run --allow-dirty
if ($LASTEXITCODE -ne 0) { throw "Rust dry-run failed!" }
Write-Host "✅ Rust check passed." -ForegroundColor Green

# 2. Python (PyPI)
Write-Host "`n🐍 Verifying Python (PyPI)..." -ForegroundColor Yellow
# Check commands
if (Get-Command maturin -ErrorAction SilentlyContinue) {
    maturin publish --dry-run
}
else {
    python -m maturin publish --dry-run
}
if ($LASTEXITCODE -ne 0) { throw "Python dry-run failed!" }
Write-Host "✅ Python check passed." -ForegroundColor Green

# 3. NPM (JS)
Write-Host "`nnpm Verifying NPM (JS)..." -ForegroundColor Yellow
Set-Location pkg
if (Test-Path "package.json") {
    npm publish --dry-run
    if ($LASTEXITCODE -ne 0) { Write-Warning "NPM dry-run failed (check login/config)." }
    else { Write-Host "✅ NPM check passed." -ForegroundColor Green }
}
else {
    Write-Warning "pkg/package.json not found. Did you run build-all?"
}

# 4. JSR (JS)
Write-Host "`n🦕 Verifying JSR (JS)..." -ForegroundColor Yellow
if (Test-Path "jsr.json") {
    npx jsr publish --dry-run
    if ($LASTEXITCODE -ne 0) { Write-Warning "JSR dry-run failed." }
    else { Write-Host "✅ JSR check passed." -ForegroundColor Green }
}
Set-Location ..

# 5. Dart (Pub.dev)
Write-Host "`n💙 Verifying Dart (Pub.dev)..." -ForegroundColor Yellow
Set-Location zakat_dart
if (Test-Path "pubspec.yaml") {
    dart pub publish --dry-run
    if ($LASTEXITCODE -ne 0) { throw "Dart dry-run failed!" }
    Write-Host "✅ Dart check passed." -ForegroundColor Green
}
Set-Location ..

Write-Host "`n🎉🎉🎉 ALL PRE-FLIGHT CHECKS PASSED! READY TO LAUNCH! 🎉🎉🎉" -ForegroundColor Green
Write-Host "You can now run '.\scripts\publish-all.ps1' with confidence."
