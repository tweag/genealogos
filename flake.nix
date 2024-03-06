{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixtract.url = "github:tweag/nixtract/snake_case-descriptions";
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , naersk
    , nixtract
    }:
    utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        cyclonedx = pkgs.callPackage ./nix/cyclonedx.nix { };
        nixtract-cli = nixtract.defaultPackage.${system};
      in
      rec {
        packages = rec {
          default = genealogos;
          genealogos = naersk-lib.buildPackage {
            src = ./.;
            doCheck = true;

            # Genealogos uses the reqwest crate to query for narinfo on the substituters.
            # reqwest depends on openssl.
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ];

            # Setting this feature flag to disable tests that require recursive nix/an internet connection
            cargoTestOptions = x: x ++ [ "--features" "nix" ];
          };
          update-fixture-output-files = pkgs.writeShellApplication {
            name = "update-fixture-output-files";
            runtimeInputs = [ (genealogos.overrideAttrs (_: { doCheck = false; })) pkgs.jq ];
            text = builtins.readFile ./scripts/update-fixture-output-files.sh;
          };
          update-fixture-input-files = pkgs.writeShellApplication {
            name = "update-fixture-input-files";
            runtimeInputs = [ nixtract-cli ];
            text = builtins.readFile ./scripts/update-fixture-input-files.sh;
          };
          verify-fixture-files = pkgs.writeShellApplication {
            name = "verify-fixture-files";
            runtimeInputs = [ cyclonedx ];
            text = builtins.readFile ./scripts/verify-fixture-files.sh;
          };
        };
        devShells = {
          default =
            pkgs.mkShell {
              buildInputs = with pkgs; [
                cargo
                cargo-dist
                rust-analyzer
                rustPackages.clippy
                rustc
                rustfmt

                nixtract-cli

                pkg-config
                openssl
              ];
              RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
              RUST_BACKTRACE = 1;
            };
          scripts =
            pkgs.mkShell {
              buildInputs = with packages; [
                update-fixture-input-files
                update-fixture-output-files
                verify-fixture-files
              ];
            };
        };
      });
}
