# Changelog

All notable changes to Torio are documented here.

## [0.1.1] - 2026-09-19

### Bug Fixes
- **project**: Fix(project) Fixing release issues ([`ff12a1f`](https://github.com/thrustlang/torio/commit/ff12a1fec5ad5ea0bb4cde77822ea0d43b07ba2c))
- **project-visual**: Fix(project-visual) Updating the documentation. ([`0eab604`](https://github.com/thrustlang/torio/commit/0eab60401b92ddc48c1780d3645361bfe1d053a8))
- **logic**: Fix(logic) Fixing the downloader redirections. ([`b523ca8`](https://github.com/thrustlang/torio/commit/b523ca8e3c59039d6291ddf5a10e523b9b77672b))



## Command Line
```console
Torio creates Thrust projects, manages versioned compiler toolchains, builds executables and libraries, and runs compiled programs.

Usage: torio <COMMAND>

Commands:
  new        Create a new executable or library project
  build      Compile the current project into dist/dev or dist/release
  run        Build and run the current executable project
  thrustc    Invoke the compiler from the active toolchain directly
  toolchain  Install, update, select, list or remove compiler toolchains
  version    Print the installed Torio version
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
  torio new hello
  torio build --release
  torio run -- argument
  torio thrustc --version
  torio toolchain install
```

---
*Torio Changelog*
