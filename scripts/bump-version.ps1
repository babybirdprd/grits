#!/usr/bin/env pwsh
# Version bump script for Grits
# Usage: ./scripts/bump-version.ps1 [major|minor|patch]
# Or:    ./scripts/bump-version.ps1 -SetVersion "1.0.8"

param(
    [ValidateSet("major", "minor", "patch")]
    [string]$BumpType,
    
    [string]$SetVersion
)

$ErrorActionPreference = "Stop"

# Get current version from grits-core/Cargo.toml (source of truth)
$coreCargoPath = "grits-core/Cargo.toml"
$coreCargoContent = Get-Content $coreCargoPath -Raw
$versionMatch = [regex]::Match($coreCargoContent, '^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', [System.Text.RegularExpressions.RegexOptions]::Multiline)

if (-not $versionMatch.Success) {
    Write-Error "Could not find version in $coreCargoPath"
    exit 1
}

$major = [int]$versionMatch.Groups[1].Value
$minor = [int]$versionMatch.Groups[2].Value
$patch = [int]$versionMatch.Groups[3].Value
$oldVersion = "$major.$minor.$patch"

# Determine new version
if ($SetVersion) {
    $newVersion = $SetVersion
    Write-Host "Setting version to: $newVersion" -ForegroundColor Cyan
} elseif ($BumpType) {
    switch ($BumpType) {
        "major" { $major++; $minor = 0; $patch = 0 }
        "minor" { $minor++; $patch = 0 }
        "patch" { $patch++ }
    }
    $newVersion = "$major.$minor.$patch"
    Write-Host "Bumping version: $oldVersion -> $newVersion" -ForegroundColor Cyan
} else {
    Write-Error "Must specify either -BumpType (major|minor|patch) or -SetVersion '1.0.8'"
    exit 1
}

# Update grits-core/Cargo.toml
$coreCargoContent = $coreCargoContent -replace '(?m)^version\s*=\s*"[\d.]+"', "version = `"$newVersion`""
Set-Content $coreCargoPath $coreCargoContent -NoNewline
Write-Host "  Updated: grits-core/Cargo.toml" -ForegroundColor Green

# Update grits-cli/Cargo.toml (version + dependency)
$cliCargoPath = "grits-cli/Cargo.toml"
$cliCargoContent = Get-Content $cliCargoPath -Raw
$cliCargoContent = $cliCargoContent -replace '(?m)^version\s*=\s*"[\d.]+"', "version = `"$newVersion`""
$cliCargoContent = $cliCargoContent -replace 'grits-core\s*=\s*\{\s*version\s*=\s*"[\d.]+"', "grits-core = { version = `"$newVersion`""
Set-Content $cliCargoPath $cliCargoContent -NoNewline
Write-Host "  Updated: grits-cli/Cargo.toml" -ForegroundColor Green

# Update README.md
$readmePath = "README.md"
$readmeContent = Get-Content $readmePath -Raw
$readmeContent = $readmeContent -replace '\(v[\d.]+\)', "(v$newVersion)"
Set-Content $readmePath $readmeContent -NoNewline
Write-Host "  Updated: README.md" -ForegroundColor Green

Write-Host ""
Write-Host "All versions set to $newVersion" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Review changes: git diff"
Write-Host "  2. Commit: git add -A && git commit -m 'chore: release v$newVersion'"
Write-Host "  3. Tag: git tag v$newVersion"
Write-Host "  4. Push: git push origin main --tags"
