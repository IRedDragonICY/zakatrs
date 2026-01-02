$ErrorActionPreference = "Stop"

Write-Host "💙 Publishing to Pub.dev..." -ForegroundColor Yellow

if (Test-Path "zakat_dart") {
    Push-Location zakat_dart
    dart pub publish
    if ($LASTEXITCODE -ne 0) { Write-Warning "Dart publish failed." }
    Pop-Location
}
else {
    Write-Warning "zakat_dart/ directory not found."
}

Write-Host "✅ Dart publish complete." -ForegroundColor Green
