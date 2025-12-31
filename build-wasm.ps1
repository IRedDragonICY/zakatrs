# build-wasm.ps1
Write-Host "🏗️  Building WASM package..."
wasm-pack build --target nodejs --scope islamic

Write-Host "📦 Restoring JSR configuration..."
Copy-Item "jsr-config\jsr.json" "pkg\jsr.json" -Force
Copy-Item "jsr-config\mod.ts" "pkg\mod.ts" -Force
Copy-Item "README.md" "pkg\README.md" -Force
Copy-Item "docs" "pkg\docs" -Recurse -Force

Write-Host "✅ Build complete! Ready to publish in ./pkg"
