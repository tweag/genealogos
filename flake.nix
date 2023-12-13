{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};
    in {
      defaultPackage = naersk-lib.buildPackage {
        src = ./.;
      };
      devShell = with pkgs;
        mkShell {
          buildInputs = [cargo rustc rustfmt rustPackages.clippy rust-analyzer];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          RUST_BACKTRACE = 1;
        };
    });
}
