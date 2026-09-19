# Releasing Torio

Torio uses `git-cliff` and the scripts under `scripts/` to prepare one release commit and four platform tags.

## Requirements

- Git with a remote named `origin`.
- Rust and Cargo.
- `git-cliff`.
- PowerShell when using the Batch script.
- A clean working tree.
- An existing previous tag. As with `thrustc`, the first release requires creating a base tag beforehand.

## Scripts

```console
bash scripts/release-changelog.sh
```

```powershell
powershell -ExecutionPolicy Bypass -File scripts/release-changelog.ps1
```

```fish
fish scripts/release-changelog.fish
```

```console
scripts\release-changelog.bat
```

Each script asks for an existing previous tag and a new full Torio tag. The new tag may use any supported platform because the script extracts only the release channel and version.

Stable example:

```text
torio-x86_64-linux-ubuntu-v0.2.0
```

Development example:

```text
torio-x86_64-linux-ubuntu-dev-v0.2.0
```

## Generated Release

The script:

1. Generates `changelogs/vX.Y.Z/README.md` from `<previous-tag>..HEAD`.
2. Appends the current `torio --help` output.
3. Updates `[package].version` in `Cargo.toml` and refreshes `Cargo.lock`.
4. Creates one release commit.
5. Creates coordinated tags for Linux x86_64, Windows x86_64, macOS x86_64 and macOS AArch64.
6. Asks before pushing the commit and then each platform tag separately.

GitHub does not create tag events when more than three tags are pushed at once. Publishing each of the four platform tags separately ensures that every release workflow receives its own `push` event. If push is declined, the release commit and tags remain local. The eight GitHub workflows use the canonical changelog as their release body.
