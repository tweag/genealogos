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
        packages = import ./nix/packages.nix { inherit pkgs naersk-lib cyclonedx nixtract-cli; };
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

                # https://github.com/rust-lang/cargo/issues/4463
                cargo-hack
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
