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
### Analyzing a local flake
```fish
genealogos --flake-ref /path/to/your/local/flake
```

### Analyzing `hello` from nixpkgs
```fish
genealogos --flake-ref nixpkgs --attribute-path hello
```

### Using a trace file
This section assumes you are using the latest `main` version version of [nixtract][nixtract].

```fish
nixtract --target-attribute-path hello /tmp/out && genealogos /tmp/out
```

For more `nixtract` arguments, see `nixtract --help`.

## Using Genealogos as a server
Genealogos can also run as an API server using the `genealogos-api` binary.
Currently, this API has only a single endpoint: `/api/analyze/<flake_ref>/<attribute_path>`.
By default, `genealogos-api` binds itself on `localhost:8000`.

## Building Genealogos
`nix build` or `cargo build`. A development shell is present via `nix devel`.

## Testing
Genealogos is tested against fixtures in `genealogos/tests/fixtures/nixtract/success/`.
With each `.in` file containing `nixtract` output and each `.out` file
containing the corresponding expected `genealogos` output. Running these tests
is done automatically by `nix build`, but can also manually be performed using
`cargo test`. Typically, `genealogos` output is non-deterministic (the UUID is
random, and the order of elements in lists is random), which makes testing a
little more annoying. To overcome this hurdle, when running `cargo test`, or
when setting the `GENEALOGOS_DETERMINISTIC` environment variable, the output of
`genealogos` is made deterministc. This is done by setting the UUID to all
zeroes, and sorting the `dependsOn` lists.

In order to make working with these fixtures a little nicer, the `nix
develop .#scripts` devShell provides two scripts. `verify-fixture-files`, which
verifies the `.out` files with the `cyclonedx-cli` tool to ensure `genealogos`
produces valid CycloneDX. And `update-fixture-files`, which should be ran when
an update to `genealogos` changes its output. Note that this second script
requires that the `cyclonedx-cli` tool is buildable.

[cyclonedx]: https://cyclonedx.org/
[nixtract]: https://github.com/tweag/nixtract/
