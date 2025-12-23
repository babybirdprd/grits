#!/usr/bin/env pwsh
# Version bump script for Grits
# Usage: ./scripts/bump-version.ps1 [major|minor|patch]

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("major", "minor", "patch")]
    [string]$BumpType
)

$ErrorActionPreference = "Stop"

# Get current version from grits-core/Cargo.toml
$coreCargoPath = "grits-core/Cargo.toml"
$coreCargoContent = Get-Content $coreCargoPath -Raw
$versionMatch = [regex]::Match($coreCargoContent, 'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"')

if (-not $versionMatch.Success) {
    Write-Error "Could not find version in $coreCargoPath"
    exit 1
}

$major = [int]$versionMatch.Groups[1].Value
$minor = [int]$versionMatch.Groups[2].Value
$patch = [int]$versionMatch.Groups[3].Value
$oldVersion = "$major.$minor.$patch"

# Bump version
switch ($BumpType) {
    "major" { $major++; $minor = 0; $patch = 0 }
    "minor" { $minor++; $patch = 0 }
    "patch" { $patch++ }
}

$newVersion = "$major.$minor.$patch"
Write-Host "Bumping version: $oldVersion -> $newVersion" -ForegroundColor Cyan

# Files to update
$files = @(
    @{ Path = "grits-core/Cargo.toml"; Pattern = 'version = "{0}"'; IsCargoCore = $true },
    @{ Path = "grits-cli/Cargo.toml"; Pattern = 'version = "{0}"'; IsCargoCore = $false },
    @{ Path = "extension/package.json"; Pattern = '"version": "{0}"'; IsCargoCore = $false },
    @{ Path = "README.md"; Pattern = 'v{0}'; IsCargoCore = $false }
)

foreach ($file in $files) {
    if (Test-Path $file.Path) {
        $content = Get-Content $file.Path -Raw
        
        # For Cargo.toml files, also update grits-core dependency version in grits-cli
        if ($file.Path -eq "grits-cli/Cargo.toml") {
            $content = $content -replace "grits-core = \{ version = `"$oldVersion`"", "grits-core = { version = `"$newVersion`""
        }
        
        $oldPattern = $file.Pattern -f $oldVersion
        $newPattern = $file.Pattern -f $newVersion
        $content = $content -replace [regex]::Escape($oldPattern), $newPattern
        
        Set-Content $file.Path $content -NoNewline
        Write-Host "  Updated: $($file.Path)" -ForegroundColor Green
    } else {
        Write-Host "  Skipped (not found): $($file.Path)" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "Version bumped to $newVersion" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Review changes: git diff"
Write-Host "  2. Commit: git add -A && git commit -m 'chore: release v$newVersion'"
Write-Host "  3. Tag: git tag v$newVersion"
Write-Host "  4. Push: git push origin main --tags"
