#!/bin/bash
set -e

echo -e "\033[0;36m🚀 Starting ZakatRS Master Publish...\033[0m"
echo "⚠️  Checking prerequisites..."
echo "   1. Have you bumped the version in Cargo.toml?"
echo "   2. Have you run './scripts/build-all.sh'?"
echo "   3. Are you logged in to all registries (cargo, npm, dart)?"

read -p "Proceed with publishing to ALL repositories? (y/n) " confirm
if [ "$confirm" != "y" ]; then
    echo "Aborted."
    exit 1
fi

# 1. Rust (Crates.io)
echo -e "\n\033[1;33m🦀 Publishing to Crates.io...\033[0m"
cargo publish

# 2. Python (PyPI)
echo -e "\n\033[1;33m🐍 Publishing to PyPI...\033[0m"
maturin publish

# 3. NPM (JS)
echo -e "\n\033[1;33mnpm Publishing to NPM...\033[0m"
cd pkg
npm publish --access public
# 4. JSR (JS)
echo -e "\n\033[1;33m🦕 Publishing to JSR...\033[0m"
npx jsr publish
cd ..

# 5. Dart (Pub.dev)
echo -e "\n\033[1;33m💙 Publishing to Pub.dev...\033[0m"
cd zakat_dart
dart pub publish
cd ..

echo -e "\n\033[0;32m✅✅✅ GLOBAL PUBLISH COMPLETE! ✅✅✅\033[0m"
