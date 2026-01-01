#!/bin/bash
set -e

# 1. Read Truth Version from Cargo.toml
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found!"
    exit 1
fi

VERSION=$(grep -m 1 '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "\033[0;36m🎯 Source Truth: Cargo.toml version is '$VERSION'\033[0m"

# Helper Function
update_manifest() {
    local file=$1
    local pattern=$2
    local replace=$3

    if [ -f "$file" ]; then
        # Use simple sed, assuming standard formatting
        sed -i -E "s/$pattern/$replace/" "$file"
        echo -e "  \033[0;32m✅ Updated $file to $VERSION\033[0m"
    else
        echo -e "  \033[0;90m- Skipping $file (not found)\033[0m"
    fi
}

# 2. Update JSONs ("version": "x.y.z")
update_manifest "jsr-config/jsr.json" "\"version\": \".*\"" "\"version\": \"$VERSION\""
update_manifest "pkg/jsr.json" "\"version\": \".*\"" "\"version\": \"$VERSION\""
update_manifest "pkg/package.json" "\"version\": \".*\"" "\"version\": \"$VERSION\""

# 3. Update Dart Pubspec (version: x.y.z)
update_manifest "zakat_dart/pubspec.yaml" "^version: .*" "version: $VERSION"

# 4. Update README.md (Dependency Examples)
# Pattern 1: zakat = "x.y.z"
update_manifest "README.md" "zakat = \".*\"" "zakat = \"$VERSION\""

# Pattern 2: zakat = { version = "x.y.z"
update_manifest "README.md" "zakat = { version = \".*\"" "zakat = { version = \"$VERSION\""

echo "🔄 Version synchronization complete."
