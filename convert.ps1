# Convert zakat_dart to a self-contained plugin

$Root = "zakat_dart"
$Builder = "$Root\rust_builder"

Write-Host "🔄 Migrating Cargokit..."
Copy-Item -Recurse -Force "$Builder\cargokit" "$Root\cargokit"

Write-Host "🔄 Migrating Platform Configs..."
$Platforms = @("android", "ios", "linux", "macos", "windows")

foreach ($p in $Platforms) {
    if (Test-Path "$Builder\$p") {
        Write-Host "  - Overwriting $p..."
        if (Test-Path "$Root\$p") {
            Remove-Item -Recurse -Force "$Root\$p"
        }
        Copy-Item -Recurse -Force "$Builder\$p" "$Root\$p"
    }
}

Write-Host "✅ Migration complete. Please update pubspec.yaml."
