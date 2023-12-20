# Genealogos
The Genealogos project is a tool that takes output from Nix evaluation tools
and produces SBOM files. Currently, it takes input from [nixtract][nixtract]
and produces json output compliant with the [CycloneDX][cyclonedx] 1.5
specification. Output from Genealogos can be used by various other tools to
perform further analysis. Any tool that takes JSON in the CycloneDX format
should accept Genealogos' output.

The project is still very early stages, so the output may as of yet be of little
use.

## Using Genealogos
This section assumes you are using the latest `main` version version of [nixtract][nixtract].

### Analyzing a package from your system nixpkgs channel
```fish
nixtract --target-attribute-path hello - | genealogos
```

### Analyzing a local flake
```fish
nixtract --target-flake-ref /path/to/your/local/flake - | genealogos
```

For more `nixtract` arguments, see `nixtract --help`.

## Building Genealogos
`nix build` or `cargo build`. A development shell is present via `nix devel`.

[cyclonedx]: https://cyclonedx.org/
[nixtract]: https://github.com/tweag/nixtract/
