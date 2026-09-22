# Changelog

All notable changes to Torio are documented here.

## [0.1.2] - 2026-09-22

### Bug Fixes
- **logic**: Fix(logic) Clarifying some messages in new command. ([`4eca340`](https://github.com/thrustlang/torio/commit/4eca340e676845c873aa3ed124cd2fc3ce0abfd9))
- **project**: Fix(project) Fixiing inquire in the script release changelog. ([`6934571`](https://github.com/thrustlang/torio/commit/693457126d591e2d39262bc524c8f36dbce4d670))


### Features
- **logic**: Feat(logic) Updating the torio toolchain installation process with animations ([`7407d97`](https://github.com/thrustlang/torio/commit/7407d9732f65c8927a78f3ea77b76c47113d1684))



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
