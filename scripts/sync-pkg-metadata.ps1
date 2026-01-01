# Sync JS/WASM Metadata Script
# Copies README, LICENSE, and JSR config to pkg/ directory.

$ErrorActionPreference = "Stop"

Write-Host "📦 Syncing JS/WASM Metadata..." -ForegroundColor Cyan

# Ensure pkg directory exists (it might be missing if wasm-pack was skipped)
if (-not (Test-Path "pkg")) {
    New-Item -ItemType Directory -Force -Path "pkg" | Out-Null
}

# Copy JSR Config
Copy-Item "jsr-config\jsr.json" "pkg\jsr.json" -Force
Copy-Item "jsr-config\mod.ts" "pkg\mod.ts" -Force

# Copy Root Metadata
Copy-Item "README.md" "pkg\README.md" -Force
Copy-Item "LICENSE" "pkg\LICENSE" -Force

# Copy Documentation
if (Test-Path "pkg\docs") { Remove-Item "pkg\docs" -Recurse -Force }
Copy-Item "docs" "pkg\docs" -Recurse -Force

Write-Host "✅ pkg/ metadata synced." -ForegroundColor Green
