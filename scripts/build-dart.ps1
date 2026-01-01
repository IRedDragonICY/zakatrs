# Automation for preparing zakat_dart package

Write-Host "📦 Syncing Documentation to zakat_dart..."

# Copy README
Copy-Item "README.md" "zakat_dart\README.md" -Force
Write-Host "✅ README.md synced."

# Copy Docs (Renamed to 'doc' for Dart standard)
# Clean up old 'docs' if it exists
if (Test-Path "zakat_dart\docs") {
    Remove-Item "zakat_dart\docs" -Recurse -Force
}
if (Test-Path "zakat_dart\doc") {
    Remove-Item "zakat_dart\doc" -Recurse -Force
}
Copy-Item "docs" "zakat_dart\doc" -Recurse -Force
Write-Host "✅ docs/ synced to doc/."

# Copy License
Copy-Item "LICENSE" "zakat_dart\LICENSE" -Force
Write-Host "✅ LICENSE synced."

# Copy Changelog
Copy-Item "CHANGELOG.md" "zakat_dart\CHANGELOG.md" -Force
Write-Host "✅ CHANGELOG synced."

Write-Host "✨ Ready to publish! Go to ./zakat_dart and run 'dart pub publish'"
