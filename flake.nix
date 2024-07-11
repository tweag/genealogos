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
        crane-lib = crane.mkLib nixpkgs.legacyPackages.${system};
        cyclonedx = pkgs.cyclonedx-cli;
        nixtract-cli = nixtract.defaultPackage.${system};

        crane-outputs = import ./nix/crane.nix {
          inherit pkgs crane-lib nixtract-cli cyclonedx;
        };
        tmp = pkgs.runCommand "tmp" { } ''
          mkdir $out
          mkdir -m 1777 $out/tmp
        '';
        dockerImage = pkgs.dockerTools.buildLayeredImageWithNixDb {
          name = "genealogos";
          tag = "latest";
          contents = [ crane-outputs.packages.genealogos-api tmp ];
          config = {
            EntryPoint = [ "genealogos-api" ];
            ExposedPorts."8000" = {};
            Env = [
              "ROCKET_ADDRESS=0.0.0.0"
              "ROCKET_PORT=8000"
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];
          };
        };
      in
      rec {
        inherit (crane-outputs) checks;
        packages = crane-outputs.packages // {
          inherit dockerImage;
        };
        overlays.default = import ./nix/overlays.nix {
          inherit crane-lib;
        };
        nixosModules.default = import ./nix/genealogos-module.nix { inherit (crane-outputs.packages) genealogos-api; };
        nixosConfigurations.genealogos-test = nixpkgs.lib.nixosSystem
          {
            inherit system;
            modules = [
              ./nix/configuration.nix
              nixosModules.default
            ];
          };

        apps.default = utils.lib.mkApp {
          drv = crane-outputs.packages.genealogos-cli;
        };

        devShells.default = crane-lib.devShell {
          inherit (crane-outputs) checks;

          packages = with pkgs; [
            rust-analyzer
          ];
        };
      });
}
