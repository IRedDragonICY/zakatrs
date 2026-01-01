#!/bin/bash
set -e

echo -e "\033[0;36m📦 Syncing JS/WASM Metadata...\033[0m"

# Ensure pkg directory exists
mkdir -p pkg

# Copy JSR Config
cp -f jsr-config/jsr.json pkg/jsr.json
cp -f jsr-config/mod.ts pkg/mod.ts

# Copy Root Metadata
cp -f README.md pkg/README.md
cp -f LICENSE pkg/LICENSE

# Copy Documentation
rm -rf pkg/docs
cp -r docs pkg/docs

echo -e "\033[0;32m✅ pkg/ metadata synced.\033[0m"
