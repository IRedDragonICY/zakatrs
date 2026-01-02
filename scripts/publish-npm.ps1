$ErrorActionPreference = "Stop"

Write-Host "📦 Publishing to NPM & JSR..." -ForegroundColor Yellow

# NPM
if (Test-Path "pkg") {
    Push-Location pkg
    Write-Host "   - Checking NPM authentication..."
    
    # Check if logged in. Mute output/error, just check exit code.
    npm whoami > $null 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠️  Not logged in to NPM. Initiating login..." -ForegroundColor Yellow
        npm login
        if ($LASTEXITCODE -ne 0) {
            Write-Error "NPM Login failed. Aborting publish."
        }
    }
    else {
        Write-Host "   - Authenticated." -ForegroundColor Cyan
    }

    Write-Host "   - Publishing to NPM..."
    npm publish --access public
    if ($LASTEXITCODE -ne 0) { Write-Warning "NPM publish failed (or already exists)." }
    Pop-Location
}
else {
    Write-Warning "pkg/ directory not found. Did you run build-wasm?"
}


Write-Host "✅ NPM publish complete." -ForegroundColor Green
