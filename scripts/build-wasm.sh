#!/bin/bash
echo "🏗️  Building WASM package..."
wasm-pack build --target nodejs --scope islamic

echo "📦 Restoring JSR configuration..."
cp jsr-config/jsr.json pkg/jsr.json
cp jsr-config/mod.ts pkg/mod.ts
cp README.md pkg/README.md
cp -r docs pkg/docs

echo "✅ Build complete! Ready to publish in ./pkg"
