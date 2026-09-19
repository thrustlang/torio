<img src="https://github.com/thrustlang/.github/blob/main/assets/logos/thrustlang-logo-name.png" alt="logo" style="width: 80%; height: 80%;"></img>

# Torio Commit Conventions

Torio uses [Git Cliff](https://git-cliff.org/) to generate its changelog. Commit titles must follow the patterns configured in [`cliff.toml`](cliff.toml) to be included.

## Title

Use one of these forms:

```text
`feat(scope)` Short description.
`fix(scope)` Short description.
```

The backticks preserve the established repository style. Git Cliff removes them before parsing the title. Titles without backticks are also recognized when they start directly with `feat(scope)` or `fix(scope)`.

Types and scopes are case-sensitive. Use lowercase names and write a concise imperative summary after the closing parenthesis.

## Types

- `feat` introduces or expands behavior.
- `fix` corrects existing behavior.

Other commit types are omitted from the generated changelog by the final catch-all parser in `cliff.toml`.

## Scopes

| Scope | Use | `feat` changelog group | `fix` changelog group |
| --- | --- | --- | --- |
| `cli` | Command-line parsing, commands, flags, and user-facing CLI behavior. | Features | Bug Fixes |
| `toolchain` | Toolchain discovery, installation, activation, updates, and editor integration. | Features | Bug Fixes |
| `build` | Project compilation, linking, profiles, artifacts, and build output. | Features | Bug Fixes |
| `run` | Running compiled projects and forwarding program arguments. | Features | Bug Fixes |
| `logic` | Shared or general Torio behavior that does not fit a more specific scope. | Features | Bug Fixes |
| `release` | Release automation, packaging, tags, and publication workflows. | Project | Bug Fixes |
| `project` | Cargo metadata, repository structure, configuration, and project scaffolding. | Project | Bug Fixes |
| `project-visual` | README files and other visual or user-facing project documentation. | Documentation | Bug Fixes |
| `doc` | Technical documentation and guides. | Documentation | Bug Fixes |

Git Cliff has generic fallbacks for unknown `feat(...)` and `fix(...)` scopes, but commits should use one of the scopes above so the changelog retains a normalized scope.

## Examples

```text
`feat(toolchain)` Follow redirects when downloading release assets.
`fix(build)` Stop the spinner before printing compiler warnings.
`feat(project)` Generate a Hello World executable template.
`feat(doc)` Document the toolchain directory layout.
`fix(cli)` Preserve arguments passed after the run separator.
```

## Combined Titles

Legacy titles may combine entries inside an outer pair of parentheses:

```text
(feat(toolchain), fix(build)) Short description.
```

Avoid this form for new commits. Git Cliff places each commit in only one changelog group, so a combined title is classified by the first matching parser rather than producing separate feature and fix entries. Prefer separate commits when changes require different types or scopes.

## Description

Use the commit body to explain why the change was needed, relevant implementation details, and any important compatibility impact. Keep the title focused on one logical change.

## Changelog Behavior

- Recognized commits are grouped as Features, Bug Fixes, Project, or Documentation.
- Commits are sorted newest first within the generated changelog.
- Merge commit titles are discarded and the repository's title backticks are removed before parsing.
- Unrecognized commit titles are excluded.
- Issue references such as `#123` are linked to the Torio GitHub issue tracker.

Git Cliff only treats tags matching these platform release families as changelog versions:

```text
torio-x86_64-linux-ubuntu-vX.Y.Z
torio-x86_64-windows-msvc-vX.Y.Z
torio-x86_64-macos-vX.Y.Z
torio-aarch64-macos-vX.Y.Z
```

Development tags use the same platform names with `-dev` before the version, for example `torio-x86_64-linux-ubuntu-dev-v0.2.0`.
