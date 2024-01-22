{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixtract.url = "github:tweag/nixtract";
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , naersk
    , nixtract
    ,
    }:
    utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        cyclonedx = pkgs.callPackage ./nix/cyclonedx.nix { };
      in
      rec {
        packages = rec {
          default = genealogos;
          genealogos = naersk-lib.buildPackage {
            src = ./.;
            doCheck = true;

            nativeBuildInputs = [ pkgs.makeWrapper ];

            postInstall = ''
              wrapProgram "$out/bin/genealogos" --prefix PATH : ${pkgs.lib.makeBinPath [ nixtract.packages.${system}.default ]}
            '';
          };
          update-fixture-files = pkgs.writeShellApplication {
            name = "update-fixture-files";
            runtimeInputs = [ (genealogos.overrideAttrs (_: { doCheck = false; })) pkgs.jq ];
            text = builtins.readFile ./scripts/update-fixture-files.sh;
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
                rust-analyzer
                rustPackages.clippy
                rustc
                rustfmt

                nixtract.packages.${system}.default
              ];
              RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
              RUST_BACKTRACE = 1;
            };
          scripts =
            pkgs.mkShell {
              buildInputs = with packages; [
                update-fixture-files
                verify-fixture-files
              ];
            };
        };
      });
}
