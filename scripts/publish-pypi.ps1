$ErrorActionPreference = "Stop"

Write-Host "🐍 Publishing to PyPI..." -ForegroundColor Yellow

if (Get-Command maturin -ErrorAction SilentlyContinue) {
    maturin publish
}
else {
    python -m maturin publish
}

if ($LASTEXITCODE -ne 0) {
    Write-Warning "Python publish failed."
}
else {
    Write-Host "✅ Python publish success." -ForegroundColor Green
}
