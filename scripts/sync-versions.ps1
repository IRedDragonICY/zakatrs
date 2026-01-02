# Sync Versions Script
# Reads version from Cargo.toml and updates other package manifests.

$ErrorActionPreference = "Stop"

# 1. Read Truth Version from Cargo.toml
$CargoPath = "Cargo.toml"
if (-not (Test-Path $CargoPath)) { throw "Cargo.toml not found!" }
$CargoContent = Get-Content $CargoPath -Raw
$VersionMatch = [regex]::match($CargoContent, 'version\s*=\s*"(.*?)"')
if (-not $VersionMatch.Success) { throw "Could not determine version from Cargo.toml" }
$Version = $VersionMatch.Groups[1].Value

Write-Host "🎯 Source Truth: Cargo.toml version is '$Version'" -ForegroundColor Cyan

# Helper Function to Update File
function Update-Manifest {
    param (
        [string]$Path,
        [string]$RegexPattern,
        [string]$ReplacementTemplate
    )

    if (Test-Path $Path) {
        $Content = Get-Content $Path -Raw
        # Check if version is already correct to avoid unnecessary writes
        if ($Content -match $RegexPattern -and $Matches[1] -eq $Version) {
            Write-Host "  - $Path is already up to date." -ForegroundColor Gray
            return
        }
        
        $NewContent = [regex]::replace($Content, $RegexPattern, $ReplacementTemplate)
        Set-Content -Path $Path -Value $NewContent -NoNewline
        Write-Host "  ✅ Updated $Path to $Version" -ForegroundColor Green
    }
    else {
        Write-Host "  - Skipping $Path (not found)" -ForegroundColor Gray
    }
}

# 2. Update JSR/Package JSONs (common JSON pattern)
# Pattern: "version": "x.y.z"
$JsonPattern = '"version":\s*"(.*?)"'
$JsonReplace = '"version": "' + $Version + '"'

Update-Manifest "jsr-config/jsr.json" $JsonPattern $JsonReplace
Update-Manifest "pkg/jsr.json" $JsonPattern $JsonReplace
Update-Manifest "pkg/package.json" $JsonPattern $JsonReplace

# 3. Update Dart Pubspec
# Pattern: version: x.y.z
$YamlPattern = 'version:\s*(.+)'
$YamlReplace = 'version: ' + $Version

Update-Manifest "zakat_dart/pubspec.yaml" $YamlPattern $YamlReplace

Update-Manifest "zakat_dart/pubspec.yaml" $YamlPattern $YamlReplace

# 4. Update README.md (Dependency Examples)
# Pattern 1: zakat = "x.y.z"
$ReadmePattern1 = 'zakat\s*=\s*"(.*?)"'
$ReadmeReplace1 = 'zakat = "' + $Version + '"'
Update-Manifest "README.md" $ReadmePattern1 $ReadmeReplace1

# Pattern 2: zakat = { version = "x.y.z"
$ReadmePattern2 = 'zakat\s*=\s*\{\s*version\s*=\s*"(.*?)"'
$ReadmeReplace2 = 'zakat = { version = "' + $Version + '"'
Update-Manifest "README.md" $ReadmePattern2 $ReadmeReplace2

# 5. Update pyproject.toml (Python)
# Pattern: version = "x.y.z" (dynamically updated by maturin usually, but good to check if static)
# Note: maturin uses Cargo.toml version dynamicially, but if we have [project] version, update it.
# Check if version is static in pyproject.toml
if (Select-String -Path "pyproject.toml" -Pattern 'version = "(.*?)"' -Quiet) {
    Update-Manifest "pyproject.toml" 'version\s*=\s*"(.*?)"' ('version = "' + $Version + '"')
}

Write-Host "🔄 Version synchronization complete."
