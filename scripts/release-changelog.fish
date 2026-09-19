#!/usr/bin/env fish
# Copyright (C) 2026  Stevens Benavides
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

cd (dirname (status filename))/..

for command_name in git git-cliff cargo
    if not type -q $command_name
        echo "Error: Required command '$command_name' was not found."
        exit 1
    end
end

if test -n "$(git status --porcelain)"
    echo "Error: The working tree must be clean before preparing a release."
    exit 1
end

git fetch --tags origin; or exit 1

echo "Available tags:"
git tag --sort=-version:refname
echo

read -P "Enter previous tag: " prev_tag

if test -z "$prev_tag"
    echo "Error: Previous tag is required."
    exit 1
end

if not git rev-parse --verify --quiet "refs/tags/$prev_tag" >/dev/null
    echo "Error: Tag '$prev_tag' does not exist."
    exit 1
end

read -P "Enter new Torio tag: " tag_name

set tag_match (string match -r '^torio-(x86_64-linux-ubuntu|x86_64-windows-msvc|x86_64-macos|aarch64-macos)(-dev)?-v([0-9]+\.[0-9]+\.[0-9]+)$' "$tag_name")

if test (count $tag_match) -eq 0
    echo "Error: Tag '$tag_name' does not match a supported Torio stable or development tag."
    exit 1
end

set release_version $tag_match[-1]
set channel ""

if string match -q '*-dev-v*' "$tag_name"
    set channel -dev
end

set range "$prev_tag..HEAD"
set release_dir "changelogs/v$release_version"
set tags \
    "torio-x86_64-linux-ubuntu$channel-v$release_version" \
    "torio-x86_64-windows-msvc$channel-v$release_version" \
    "torio-x86_64-macos$channel-v$release_version" \
    "torio-aarch64-macos$channel-v$release_version"

for release_tag in $tags
    if git rev-parse --verify --quiet "refs/tags/$release_tag" >/dev/null
        echo "Error: Local tag '$release_tag' already exists."
        exit 1
    end

    if git ls-remote --exit-code --tags origin "refs/tags/$release_tag" >/dev/null 2>&1
        echo "Error: Remote tag '$release_tag' already exists."
        exit 1
    end
end

mkdir -p "$release_dir"

echo "Generating changelog for: $range"
git-cliff "$range" --tag "v$release_version" --output "$release_dir/README.md"; or exit 1

cargo build --quiet; or exit 1
set help_output (./target/debug/torio --help 2>&1 | string collect)

if test -n "$help_output"
    echo "" >> "$release_dir/README.md"
    echo "## Command Line" >> "$release_dir/README.md"
    echo '```console' >> "$release_dir/README.md"
    echo "$help_output" >> "$release_dir/README.md"
    echo '```' >> "$release_dir/README.md"
end

echo "" >> "$release_dir/README.md"
echo "---" >> "$release_dir/README.md"
echo "*Torio Changelog*" >> "$release_dir/README.md"

awk -v ver="$release_version" '
    /^\[package\]/ { inpkg=1; print; next }
    /^\[/ { inpkg=0; print; next }
    inpkg && /^version[[:space:]]*=/ { print "version = \"" ver "\""; next }
    { print }
' Cargo.toml > Cargo.toml.tmp; and mv Cargo.toml.tmp Cargo.toml; or exit 1

cargo check --quiet; or exit 1

git add "$release_dir/README.md" Cargo.toml Cargo.lock
git commit -m "Bumping 'v$release_version'"; or exit 1

for release_tag in $tags
    git tag "$release_tag"; or exit 1
end

echo "Prepared tags:"
printf '  %s\n' $tags

read -P "Push the release commit and all tags to origin? [y/N] " push_answer

if string match -rq '^[Yy]$' "$push_answer"
    git push --atomic origin HEAD $tags; or exit 1
    echo "Release commit and tags pushed to origin."
else
    echo "Push skipped. The release commit and tags remain local."
end

echo "Changelog generated at $release_dir/README.md"
