# This nix file contains all our own packages!
{ pkgs, naersk-lib, nixtract-cli, cyclonedx }:
let
  buildRustPackage =
    { name } @ args: naersk-lib.buildPackage (args // {
      src = ../.;
      doCheck = true;

      cargoBuildOptions = x: x ++ [ "--package" name ];

      # Setting this feature flag to disable tests that require recursive nix/an internet connection
      cargoTestOptions = x: x ++ [ "--package" name "--features" "nix" ];

      # Genealogos uses the reqwest crate to query for narinfo on the substituters.
      # reqwest depends on openssl.
      nativeBuildInputs = with pkgs; [ pkg-config ];
      buildInputs = with pkgs; [ openssl ];
    });
in
rec {
  default = genealogos;
  genealogos = buildRustPackage {
    name = "genealogos";
  };
  genealogos-cli = buildRustPackage {
    name = "genealogos-cli";
  };
  genealogos-api = buildRustPackage {
    name = "genealogos-api";
  };
  update-fixture-output-files = pkgs.writeShellApplication {
    name = "update-fixture-output-files";
    runtimeInputs = [ (genealogos-cli.overrideAttrs (_: { doCheck = false; })) pkgs.jq ];
    text = builtins.readFile ../scripts/update-fixture-output-files.sh;
  };
  update-fixture-input-files = pkgs.writeShellApplication {
    name = "update-fixture-input-files";
    runtimeInputs = [ nixtract-cli ];
    text = builtins.readFile ../scripts/update-fixture-input-files.sh;
  };
  verify-fixture-files = pkgs.writeShellApplication {
    name = "verify-fixture-files";
    runtimeInputs = [ cyclonedx ];
    text = builtins.readFile ../scripts/verify-fixture-files.sh;
  };

  # This is a special package that we use mainly for CI.
  # It uses naersk to build the entire rust workspace.
  all = naersk-lib.buildPackage {
    name = "genealogos";
    src = ../.;
    doCheck = true;
    cargoOptions = x: x ++ [ "hack" ];
    cargoTestOptions = x: x ++ [ "--features" "nix" ];
    nativeBuildInputs = with pkgs; [ pkg-config cargo-hack ];
    buildInputs = with pkgs; [ openssl ];
    RUST_BACKTRACE = 1;
  };
}
