# Changelog

All notable changes to Torio are documented here.

## [0.1.3] - 2026-10-02

### Documentation
- **project-visual**: Feat(project-visual) Adding a guide for downloading and install torio in github releases ([`69b7dd2`](https://github.com/thrustlang/torio/commit/69b7dd2c2ad2257f796494380b11d282a15023bb))


### Project
- **project**: Feat(project) Adding C runtime static linking in windows. ([`090066a`](https://github.com/thrustlang/torio/commit/090066aa75b2333a408ed72e578d68f501df7d8b))



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
