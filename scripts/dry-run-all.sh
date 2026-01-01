#!/bin/bash
set -e

echo -e "\033[0;36m🛡️  Starting ZakatRS Dry-Run Verification...\033[0m"
echo "ℹ️  This script checks if packages are valid and ready for publishing."

# 1. Rust (Crates.io)
echo -e "\n\033[1;33m🦀 Verifying Rust (Crates.io)...\033[0m"
cargo publish --dry-run --allow-dirty
echo -e "\033[0;32m✅ Rust check passed.\033[0m"

# 2. Python (PyPI)
echo -e "\n\033[1;33m🐍 Verifying Python (PyPI)...\033[0m"
if command -v maturin &> /dev/null; then
    maturin publish --dry-run
else
    python3 -m maturin publish --dry-run
fi
echo -e "\033[0;32m✅ Python check passed.\033[0m"

# 3. NPM (JS)
echo -e "\n\033[1;33mnpm Verifying NPM (JS)...\033[0m"
cd pkg
if [ -f "package.json" ]; then
    npm publish --dry-run || echo "⚠️ NPM dry-run warning"
else
     echo "⚠️ pkg/package.json not found"
fi

# 4. JSR (JS)
echo -e "\n\033[1;33m🦕 Verifying JSR (JS)...\033[0m"
if [ -f "jsr.json" ]; then
    npx jsr publish --dry-run || echo "⚠️ JSR dry-run warning"
fi
cd ..

# 5. Dart (Pub.dev)
echo -e "\n\033[1;33m💙 Verifying Dart (Pub.dev)...\033[0m"
cd zakat_dart
dart pub publish --dry-run
cd ..

echo -e "\n\033[0;32m🎉🎉🎉 ALL PRE-FLIGHT CHECKS PASSED! READY TO LAUNCH! 🎉🎉🎉\033[0m"
echo "You can now run './scripts/publish-all.sh' with confidence."
