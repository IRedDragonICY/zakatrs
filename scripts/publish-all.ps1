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
.\scripts\publish-crates.ps1

# 2. Python (PyPI)
.\scripts\publish-pypi.ps1

# 3. NPM & JSR (JS)
.\scripts\publish-npm.ps1

# 4. Dart (Pub.dev)
.\scripts\publish-dart.ps1

Write-Host "`n✅✅✅ GLOBAL PUBLISH COMMANDS EXECUTED! ✅✅✅" -ForegroundColor Green
