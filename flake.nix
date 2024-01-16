{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , naersk
    ,
    }:
    utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        cyclonedx = pkgs.callPackage ./nix/cyclonedx.nix { };
      in
      {
        packages = {
          default = naersk-lib.buildPackage {
            src = ./.;
            doCheck = true;
          };
          inherit cyclonedx;
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
              ];
              RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
              RUST_BACKTRACE = 1;
            };
        };
      });
}
