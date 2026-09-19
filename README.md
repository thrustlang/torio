<img src= "https://github.com/thrustlang/.github/blob/main/assets/logos/new%20logo/thrustlang-logo-banner-text-italic.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# The Thrust Package Manager 

<img src= "https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt= "standard-separator" style= "width: 1hv;"> </img>

Torio is the high-level project and toolchain manager for the Thrust compiler. It installs versioned compiler and LSP components, creates projects, and drives reproducible builds through `thrustc`.

## Commands

```console
torio toolchain install [version]
torio toolchain update
torio toolchain list
torio toolchain use <version>
torio toolchain remove <version>
torio new <name> [--executable|--lib]
torio build [--release] [--cc-arg <argument>]
torio run [--release] [--cc-arg <argument>] [-- <program arguments>]
```

Projects are configured through `torio.yml`.
