#!/bin/bash
set -e

echo "📦 Syncing Documentation to zakat_dart..."

# Copy README
cp README.md zakat_dart/README.md
echo "✅ README.md synced."

# Copy Docs (Renamed to 'doc' for Dart standard)
rm -rf zakat_dart/docs
rm -rf zakat_dart/doc
cp -r docs zakat_dart/doc
echo "✅ docs/ synced to doc/."

# Copy License
cp LICENSE zakat_dart/LICENSE
echo "✅ LICENSE synced."

# Copy Changelog
cp CHANGELOG.md zakat_dart/CHANGELOG.md
echo "✅ CHANGELOG synced."

echo "✨ Ready to publish! Go to ./zakat_dart and run 'dart pub publish'"
