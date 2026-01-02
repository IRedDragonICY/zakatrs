$ErrorActionPreference = "Stop"

Write-Host "🦀 Publishing to Crates.io..." -ForegroundColor Yellow
cargo publish

if ($LASTEXITCODE -ne 0) { 
    Write-Warning "Rust publish failed (maybe already published?)" 
}
else {
    Write-Host "✅ Rust publish success." -ForegroundColor Green
}
