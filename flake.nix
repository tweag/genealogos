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

        crane-outputs = import ./nix/crane.nix {
          inherit pkgs crane-lib nixtract-cli cyclonedx;
        };
      in
      rec {
        inherit (crane-outputs) checks packages;
        overlays.default = import ./nix/overlays.nix {
          inherit crane-lib;
        };
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

        apps.default = utils.lib.mkApp {
          drv = crane-outputs.packages.genealogos-cli;
        };


        devShells.default = crane-lib.devShell {
          inherit (crane-outputs) checks;

          packages = with pkgs; [
            rust-analyzer

            # https://github.com/rust-lang/cargo/issues/4463
            cargo-hack
          ];
        };
      });
}
