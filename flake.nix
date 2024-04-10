{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixtract.url = "github:tweag/nixtract";
  };

  outputs =
    { self
    , crane
    , nixpkgs
    , utils
    , nixtract
    }:
    utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = import nixpkgs { inherit system; };
        crane-lib = crane.lib.${system};
        cyclonedx = pkgs.callPackage ./nix/cyclonedx.nix { };
        nixtract-cli = nixtract.defaultPackage.${system};

        # Here we start the crane stuff
        common-crane-args = {
          pname = "genealogos";
          src = crane-lib.cleanCargoSource (crane-lib.path ./.);
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
        overlays.default = import ./nix/overlays.nix { inherit pkgs packages; };
        nixosModules.default = import ./nix/genealogos-module.nix;
        nixosConfigurations.genealogos = nixpkgs.lib.nixosSystem
          {
            pkgs = import nixpkgs { inherit system; overlays = [ overlays.default ]; };
            inherit system;
            modules = [
              ./nix/configuration.nix
              ./nix/genealogos-module.nix
            ];
          };
        packages =
          rust-packages // {
            default = packages.genealogos;

            workspace = crane-lib.buildPackage workspace;

            update-fixture-output-files = pkgs.writeShellApplication {
              name = "update-fixture-output-files";
              runtimeInputs = [ (packages.genealogos-cli.overrideAttrs (_: { doCheck = false; })) pkgs.jq ];
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

        apps.default = utils.lib.mkApp {
          drv = packages.genealogos-cli;
        };


        devShells.default = crane-lib.devShell {
          inherit checks;

          packages = with pkgs; [
            rust-analyzer

            # https://github.com/rust-lang/cargo/issues/4463
            cargo-hack
          ];
        };
      });
}
