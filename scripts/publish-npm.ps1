$ErrorActionPreference = "Stop"

Write-Host "📦 Publishing to NPM & JSR..." -ForegroundColor Yellow

# NPM
if (Test-Path "pkg") {
    Push-Location pkg
    Write-Host "   - Publishing to NPM..."
    npm publish --access public
    if ($LASTEXITCODE -ne 0) { Write-Warning "NPM publish failed (or already exists)." }
    Pop-Location
}
else {
    Write-Warning "pkg/ directory not found. Did you run build-wasm?"
}

# JSR
Write-Host "   - Publishing to JSR..."
npx jsr publish
if ($LASTEXITCODE -ne 0) { Write-Warning "JSR publish failed." }

Write-Host "✅ JS publish complete." -ForegroundColor Green
