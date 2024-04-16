# This file contains everything related to building our packages with crane.
# It returns a few things; the build packages and the checks
{ pkgs
, crane-lib
, nixtract-cli ? null
, cyclonedx ? null
}:
let
  common-crane-args = {
    pname = "genealogos";

    # We need to also include the frontend .html file for the include_str macro in the api
    src = pkgs.lib.cleanSourceWith {
      src = crane-lib.path ../.;
      filter = path: type: (crane-lib.filterCargoSources path type) || (builtins.match ".*/genealogos-frontend/index.html" path != null);
    };

    strictDeps = true;

    cargoArtifacts = cargo-artifacts;

    # Genealogos uses the reqwest crate to query for narinfo on the substituters.
    # reqwest depends on openssl.
    nativeBuildInputs = with pkgs; [ pkg-config ];
    buildInputs = with pkgs; [ openssl ];
  };

  cargo-artifacts = crane-lib.buildDepsOnly common-crane-args;

  workspace = (common-crane-args // {
    cargoBuildCommand = "${pkgs.cargo-hack}/bin/cargo-hack hack build --profile release";
    cargoTestCommand = "${pkgs.cargo-hack}/bin/cargo-hack hack test --profile release";
  });

  # Crane buildPackage arguments for every crate
  crates = {
    genealogos = (common-crane-args // {
      cargoExtraArgs = "-p genealogos";
    });
    genealogos-cli = (common-crane-args // {
      pname = "genealogos-cli";
      cargoExtraArgs = "-p genealogos-cli";
      passthru.exePath = "/bin/genealogos";
      nativeBuildInputs = common-crane-args.nativeBuildInputs ++ [ pkgs.makeWrapper ];
      preFixup = ''
        wrapProgram $out/bin/genealogos \
          --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.nix ]}
      '';
    });
    genealogos-api = (common-crane-args // {
      pname = "genealogos-api";
      cargoExtraArgs = "-p genealogos-api";
      nativeBuildInputs = common-crane-args.nativeBuildInputs ++ [ pkgs.makeWrapper ];
      preFixup = ''
        wrapProgram $out/bin/genealogos-api \
          --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.nix ]}
      '';
    });
  };
  rust-packages =
    builtins.mapAttrs (_: crane-lib.buildPackage) crates;
in
rec {
  checks =
    # Builds
    rust-packages
    # Clippy
    // builtins.mapAttrs
      (_: args: crane-lib.cargoClippy (args // {
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      }))
      crates
    # Doc
    // builtins.mapAttrs (_: crane-lib.cargoDoc) crates
    # fmt
    // builtins.mapAttrs (_: crane-lib.cargoFmt) crates;

  packages =
    rust-packages // {
      default = packages.genealogos-cli;

      workspace = crane-lib.buildPackage workspace;

      update-fixture-output-files = pkgs.writeShellApplication {
        name = "update-fixture-output-files";
        runtimeInputs = [ (packages.genealogos-cli.overrideAttrs (_: { doCheck = false; })) pkgs.jq ];
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
    };
}
