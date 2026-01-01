# Master Publish Script for ZakatRS
# Publishes to Crates.io, PyPI, NPM, JSR, and Pub.dev.

$ErrorActionPreference = "Stop"

Write-Host "🚀 Starting ZakatRS Master Publish..." -ForegroundColor Cyan
Write-Host "⚠️  Checking prerequisites..."
Write-Host "   1. Have you bumped the version in Cargo.toml?"
Write-Host "   2. Have you run '.\scripts\build-all.ps1'?"
Write-Host "   3. Are you logged in to all registries (cargo, npm, dart)?"

$confirm = Read-Host "Proceed with publishing to ALL repositories? (y/n)"
if ($confirm -ne 'y') { Write-Host "Aborted."; exit }

# 1. Rust (Crates.io)
Write-Host "`n🦀 Publishing to Crates.io..." -ForegroundColor Yellow
cargo publish
if ($LASTEXITCODE -ne 0) { throw "Rust publish failed!" }

# 2. Python (PyPI)
Write-Host "`n🐍 Publishing to PyPI..." -ForegroundColor Yellow
maturin publish
if ($LASTEXITCODE -ne 0) { throw "Python publish failed!" }

# 3. NPM (JS)
Write-Host "`nnpm Publishing to NPM..." -ForegroundColor Yellow
Set-Location pkg
npm publish --access public
if ($LASTEXITCODE -ne 0) { Write-Warning "NPM publish failed (or already exists)." }

# 4. JSR (JS)
Write-Host "`n🦕 Publishing to JSR..." -ForegroundColor Yellow
npx jsr publish
if ($LASTEXITCODE -ne 0) { Write-Warning "JSR publish failed." }
Set-Location ..

# 5. Dart (Pub.dev)
Write-Host "`n💙 Publishing to Pub.dev..." -ForegroundColor Yellow
Set-Location zakat_dart
dart pub publish
if ($LASTEXITCODE -ne 0) { throw "Dart publish failed!" }
Set-Location ..

Write-Host "`n✅✅✅ GLOBAL PUBLISH COMPLETE! ✅✅✅" -ForegroundColor Green
