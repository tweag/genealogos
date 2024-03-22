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
`genealogos-api` provides two categories of endpoints.
A blocking endpoint and one based on jobs.

### Blocking
Currently, there is only a single blocking endpoint: `/api/analyze?flake_ref=<flake_ref>&attribute_path=<attribute_path>`.
By default, `genealogos-api` binds itself on `localhost:8000`.

For example, using curl, the api can be invoked like this:
```fish
curl "http://localhost:8000/api/analyze?flake_ref=nixpkgs&attribute_path=hello"
```

Additionally an optional `bom_format` query parameter can be provided to specify the sbom format to use.
Example:
```fish
curl "http://localhost:8000/api/analyze?flake_ref=nixpkgs&attribute_path=hello&cyclonedx_version=v1_4"
```

<!-- TODO: Add 1.5 support -->
Currently supported are `[cyclonedx_1.3_json, cyclonedx_1.3_xml, cyclonedx_1.4_json, cyclonedx_1.4_xml]`, with `cyclonedx_1.4_json` being the default.

### Jobs
The jobs based API consists of three endpoints: `/api/jobs/create`, `/api/jobs/status`, and `/api/jobs/result`.

Creating a job is done in a similar fashion to the blocking api:
```fish
curl "http://localhost:8000/api/jobs/create?flake_ref=nixpkgs&attribute_path=hello"
```
This endpoint also supports the `bom_format` query parameter.
The response of this API call is a `job_id`, which needs to be passed to further calls to indentify the desired job.

Getting the status of a job is done as such:
```fish
curl "http://localhost:8000/api/jobs/status/0"
```
where 0 was the `job_id` provided in the previous call.
This API can return one of `stopped`, `running` and `done`.

Finally, getting the result is done with the `result` endpoint:
```fish
curl "http://localhost:8000/api/jobs/result/0"
```

## Using the Geanlogos Web UI
Genealogos ships with a pure html/javascript web frontend.
By default, this frontend uses `127.0.0.1` to connect to the `genealogos-api`.
Changing this default can be done using the settings button in the top of the webpage.

The Web UI currently only supports analyzing from a flake ref and attribute path, analyzing from a trace file is not yet supported.

## Building Genealogos
`nix build` or `cargo build`. A development shell is provided via `nix devel`.

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
