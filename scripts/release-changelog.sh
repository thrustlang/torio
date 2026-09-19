#!/usr/bin/env bash
#
# Copyright (C) 2026  Stevens Benavides
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

set -euo pipefail

cd "$(dirname "$0")/.."

for command in git git-cliff cargo; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "Error: Required command '$command' was not found."
        exit 1
    fi
done

if [ -n "$(git status --porcelain)" ]; then
    echo "Error: The working tree must be clean before preparing a release."
    exit 1
fi

git fetch --tags origin

echo "Available tags:"
git tag --sort=-version:refname
echo

read -rp "Enter previous tag: " prev_tag

if [ -z "$prev_tag" ]; then
    echo "Error: Previous tag is required."
    exit 1
fi

if ! git rev-parse --verify --quiet "refs/tags/${prev_tag}" >/dev/null; then
    echo "Error: Tag '${prev_tag}' does not exist."
    exit 1
fi

read -rp "Enter new Torio tag: " tag_name

tag_pattern='^torio-(x86_64-linux-ubuntu|x86_64-windows-msvc|x86_64-macos|aarch64-macos)(-dev)?-v([0-9]+\.[0-9]+\.[0-9]+)$'

if [[ ! "$tag_name" =~ $tag_pattern ]]; then
    echo "Error: Tag '${tag_name}' does not match a supported Torio stable or development tag."
    exit 1
fi

channel="${BASH_REMATCH[2]}"
version="${BASH_REMATCH[3]}"
range="${prev_tag}..HEAD"
release_dir="changelogs/v${version}"
tags=(
    "torio-x86_64-linux-ubuntu${channel}-v${version}"
    "torio-x86_64-windows-msvc${channel}-v${version}"
    "torio-x86_64-macos${channel}-v${version}"
    "torio-aarch64-macos${channel}-v${version}"
)

for release_tag in "${tags[@]}"; do
    if git rev-parse --verify --quiet "refs/tags/${release_tag}" >/dev/null; then
        echo "Error: Local tag '${release_tag}' already exists."
        exit 1
    fi

    if git ls-remote --exit-code --tags origin "refs/tags/${release_tag}" >/dev/null 2>&1; then
        echo "Error: Remote tag '${release_tag}' already exists."
        exit 1
    fi
done

mkdir -p "$release_dir"

echo "Generating changelog for: ${range}"
git-cliff "$range" --tag "v${version}" --output "${release_dir}/README.md"

cargo build --quiet
help_output=$(./target/debug/torio --help 2>&1)

if [ -n "$help_output" ]; then
    {
        echo
        echo "## Command Line"
        echo '```console'
        echo "$help_output"
        echo '```'
    } >> "${release_dir}/README.md"
fi

{
    echo
    echo "---"
    echo "*Torio Changelog*"
} >> "${release_dir}/README.md"

awk -v ver="$version" '
    /^\[package\]/ { inpkg=1; print; next }
    /^\[/ { inpkg=0; print; next }
    inpkg && /^version[[:space:]]*=/ { print "version = \"" ver "\""; next }
    { print }
' Cargo.toml > Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml

cargo check --quiet

git add "${release_dir}/README.md" Cargo.toml Cargo.lock
git commit -m "Bumping 'v${version}'"

for release_tag in "${tags[@]}"; do
    git tag "$release_tag"
done

echo "Prepared tags:"
printf '  %s\n' "${tags[@]}"

read -rp "Push the release commit and each tag separately to origin? [y/N] " push_answer

if [[ "$push_answer" =~ ^[Yy]$ ]]; then
    git push origin HEAD

    for release_tag in "${tags[@]}"; do
        git push origin "$release_tag"
    done

    echo "Release commit and tags pushed to origin."
else
    echo "Push skipped. The release commit and tags remain local."
fi

echo "Changelog generated at ${release_dir}/README.md"
