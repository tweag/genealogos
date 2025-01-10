{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
    };

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixtract.url = "github:tweag/nixtract";
  };

  outputs =
    { self
    , crane
    , nixpkgs
    , nixtract
    }:

    let
      # Matches pkgs.tree-sitter
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "i686-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      pkgsFor = nixpkgs.lib.genAttrs supportedSystems (system: import nixpkgs {
        inherit system;
      });

      forAllSystems = fn: nixpkgs.lib.genAttrs supportedSystems (system: fn rec {
        inherit system;
        pkgs = pkgsFor.${system};
        inherit (pkgs) lib;
      });

      mkGenealogosArtifacts = pkgs: rec {
        crane-lib = crane.mkLib nixpkgs.legacyPackages.${pkgs.system};
        nixtract-cli = nixtract.defaultPackage.${pkgs.system};
        crane-outputs = import ./nix/crane.nix {
          inherit pkgs crane-lib nixtract-cli;
          inherit (pkgs) cyclonedx-cli;
        };
      };
    in

    {
      overlays.default = import ./nix/overlays.nix {
        inherit crane;
      };

      nixosModules.default = import ./nix/genealogos-module.nix { inherit mkGenealogosArtifacts; };

      packages = forAllSystems ({ system, pkgs, ... }:
        let artifacts = mkGenealogosArtifacts pkgs; in
        let
          tmp = pkgs.runCommand "tmp" { } ''
            mkdir $out
            mkdir -m 1777 $out/tmp
          '';
        in
        artifacts.crane-outputs.packages // {
          dockerImage = pkgs.dockerTools.buildLayeredImageWithNixDb {
            name = "genealogos";
            tag = "latest";
            contents = [ artifacts.crane-outputs.packages.genealogos-api tmp ];
            config = {
              EntryPoint = [ "genealogos-api" ];
              ExposedPorts."8000" = { };
              Env = [
                "ROCKET_ADDRESS=0.0.0.0"
                "ROCKET_PORT=8000"
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };
        }
      );

      checks = forAllSystems ({ system, pkgs, ... }:
        let artifacts = mkGenealogosArtifacts pkgs; in
        artifacts.crane-outputs.checks
      );

      apps = forAllSystems ({ system, pkgs, ... }:
        let artifacts = mkGenealogosArtifacts pkgs; in
        {
          default = {
            type = "app";
            program =
              artifacts.crane-outputs.packages.genealogos-cli.passthru.exePath;
          };
        });

      devShells = forAllSystems ({ system, pkgs, ... }:
        let artifacts = mkGenealogosArtifacts pkgs; in
        {
          default = artifacts.crane-lib.devShell {
            inherit (artifacts.crane-outputs) checks;

            packages = with pkgs; [
              rust-analyzer
            ];
          };
        });
    };
}
