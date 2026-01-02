$ErrorActionPreference = "Stop"

Write-Host "📦 Publishing to JSR..." -ForegroundColor Yellow

if (Test-Path "jsr-config") {
    # Ensure pkg/ exists and has artifacts
    if (-not (Test-Path "pkg/zakat.js")) {
        Write-Error "pkg/zakat.js not found. Run build-wasm first."
    }

    Write-Host "   - Copying WASM artifacts to jsr-config/..."
    Copy-Item -Path "pkg/zakat.js" -Destination "jsr-config/" -Force
    Copy-Item -Path "pkg/zakat_bg.wasm" -Destination "jsr-config/" -Force
    Copy-Item -Path "pkg/zakat.d.ts" -Destination "jsr-config/" -Force
    # Copy bg type definitions if needed
    Copy-Item -Path "pkg/zakat_bg.wasm.d.ts" -Destination "jsr-config/" -Force
    
    Push-Location jsr-config
    Write-Host "   - Publishing to JSR..."
    npx jsr publish --allow-slow-types
    if ($LASTEXITCODE -ne 0) { Write-Warning "JSR publish failed." }
    Pop-Location
}
else {
    Write-Warning "jsr-config/ directory not found. Skipping JSR publish."
}

Write-Host "✅ JSR publish complete." -ForegroundColor Green
