# Changelog

All notable changes to Torio are documented here.

## [0.1.4] - 2026-10-02

### Bug Fixes
- **project**: Fix(project) Removing unnecessary comment in build.rs ([`8d318e8`](https://github.com/thrustlang/torio/commit/8d318e8e8378c080de7e4cf1777b4badde64f603))


### Project
- **project**: Feat(project) Adding relevant metadata for windows msvc executable ([`e507c24`](https://github.com/thrustlang/torio/commit/e507c24b7726ce8e6e88b7f29297674f7bbc252d))



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
