# Copyright (C) 2026  Stevens Benavides
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location (Join-Path $PSScriptRoot "..")

foreach ($command in @("git", "git-cliff", "cargo")) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "Required command '$command' was not found."
    }
}

if (-not [string]::IsNullOrWhiteSpace((git status --porcelain))) {
    throw "The working tree must be clean before preparing a release."
}

git fetch --tags origin
if ($LASTEXITCODE -ne 0) { throw "Failed to fetch tags from origin." }

Write-Host "Available tags:"
git tag --sort=-version:refname
Write-Host ""

$prevTag = Read-Host "Enter previous tag"

if ([string]::IsNullOrWhiteSpace($prevTag)) {
    throw "Previous tag is required."
}

git rev-parse --verify --quiet "refs/tags/$prevTag" *> $null
if ($LASTEXITCODE -ne 0) { throw "Tag '$prevTag' does not exist." }

$tagName = Read-Host "Enter new Torio tag"
$tagPattern = '^torio-(x86_64-linux-ubuntu|x86_64-windows-msvc|x86_64-macos|aarch64-macos)(-dev)?-v(\d+\.\d+\.\d+)$'
$tagMatch = [regex]::Match($tagName, $tagPattern)

if (-not $tagMatch.Success) {
    throw "Tag '$tagName' does not match a supported Torio stable or development tag."
}

$channel = $tagMatch.Groups[2].Value
$version = $tagMatch.Groups[3].Value
$range = "$prevTag..HEAD"
$releaseDir = "changelogs/v$version"
$tags = @(
    "torio-x86_64-linux-ubuntu$channel-v$version",
    "torio-x86_64-windows-msvc$channel-v$version",
    "torio-x86_64-macos$channel-v$version",
    "torio-aarch64-macos$channel-v$version"
)

foreach ($releaseTag in $tags) {
    git rev-parse --verify --quiet "refs/tags/$releaseTag" *> $null
    if ($LASTEXITCODE -eq 0) { throw "Local tag '$releaseTag' already exists." }

    git ls-remote --exit-code --tags origin "refs/tags/$releaseTag" *> $null
    if ($LASTEXITCODE -eq 0) { throw "Remote tag '$releaseTag' already exists." }
}

New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null

Write-Host "Generating changelog for: $range"
git-cliff $range --tag "v$version" --output "$releaseDir/README.md"
if ($LASTEXITCODE -ne 0) { throw "git-cliff failed." }

cargo build --quiet
if ($LASTEXITCODE -ne 0) { throw "Torio build failed." }

$helpOutput = & "./target/debug/torio.exe" --help 2>&1 | Out-String

if (-not [string]::IsNullOrWhiteSpace($helpOutput)) {
    Add-Content -Encoding utf8 "$releaseDir/README.md" ""
    Add-Content -Encoding utf8 "$releaseDir/README.md" "## Command Line"
    Add-Content -Encoding utf8 "$releaseDir/README.md" '```console'
    Add-Content -Encoding utf8 "$releaseDir/README.md" $helpOutput.TrimEnd()
    Add-Content -Encoding utf8 "$releaseDir/README.md" '```'
}

Add-Content -Encoding utf8 "$releaseDir/README.md" ""
Add-Content -Encoding utf8 "$releaseDir/README.md" "---"
Add-Content -Encoding utf8 "$releaseDir/README.md" "*Torio Changelog*"

$inPackage = $false
$cargoContent = Get-Content "Cargo.toml" | ForEach-Object {
    if ($_ -match '^\[package\]') { $inPackage = $true }
    elseif ($_ -match '^\[') { $inPackage = $false }

    if ($inPackage -and $_ -match '^version\s*=') {
        "version = `"$version`""
    } else {
        $_
    }
}
Set-Content -Encoding utf8 "Cargo.toml" $cargoContent

cargo check --quiet
if ($LASTEXITCODE -ne 0) { throw "Cargo validation failed." }

git add "$releaseDir/README.md" Cargo.toml Cargo.lock
git commit -m "Bumping 'v$version'"
if ($LASTEXITCODE -ne 0) { throw "Failed to create release commit." }

foreach ($releaseTag in $tags) {
    git tag $releaseTag
    if ($LASTEXITCODE -ne 0) { throw "Failed to create tag '$releaseTag'." }
}

Write-Host "Prepared tags:"
$tags | ForEach-Object { Write-Host "  $_" }

$pushAnswer = Read-Host "Push the release commit and each tag separately to origin? [y/N]"

if ($pushAnswer -match '^[Yy]$') {
    git push origin HEAD
    if ($LASTEXITCODE -ne 0) { throw "Failed to push the release commit." }

    foreach ($releaseTag in $tags) {
        git push origin $releaseTag
        if ($LASTEXITCODE -ne 0) { throw "Failed to push tag '$releaseTag'." }
    }

    Write-Host "Release commit and tags pushed to origin."
} else {
    Write-Host "Push skipped. The release commit and tags remain local."
}

Write-Host "Changelog generated at $releaseDir/README.md"
