$ErrorActionPreference = "Stop"

Write-Host "💙 Publishing to Pub.dev..." -ForegroundColor Yellow

if (Test-Path "zakat_dart") {
    # Ensure critical package files are present (they are gitignored to avoid duplication)
    Write-Host "   - Syncing README, LICENSE, CHANGELOG..."
    Copy-Item "README.md" "zakat_dart\README.md" -Force
    Copy-Item "LICENSE" "zakat_dart\LICENSE" -Force
    Copy-Item "CHANGELOG.md" "zakat_dart\CHANGELOG.md" -Force

    Push-Location zakat_dart
    dart pub publish
    if ($LASTEXITCODE -ne 0) { Write-Warning "Dart publish failed." }
    Pop-Location
}
else {
    Write-Warning "zakat_dart/ directory not found."
}

Write-Host "✅ Dart publish complete." -ForegroundColor Green
