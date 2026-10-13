# Changelog

All notable changes to Torio are documented here.

## [0.1.5] - 2026-10-13

### Project
- **project**: Feat(project) Adding a command to change the compiler variant between release and stripped. ([`58ebc2d`](https://github.com/thrustlang/torio/commit/58ebc2de2086ccd4bd0af3daf6174e8f3862bea1))



## Command Line
```console
Torio creates Thrust projects, manages versioned compiler toolchains, builds executables and libraries, and runs compiled programs.

Usage: torio <COMMAND>

Commands:
  new               Create a new executable or library project
  build             Compile the current project into dist/dev or dist/release
  run               Build and run the current executable project
  thrustc           Invoke the compiler from the active toolchain directly
  thrustc-stripped  Invoke the stripped compiler from the active toolchain directly
  toolchain         Install, update, select, list or remove compiler toolchains
  version           Print the installed Torio version
  help              Print this message or the help of the given subcommand(s)

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
