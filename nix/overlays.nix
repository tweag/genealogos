{ crane }:
final: prev:
let
  crane-lib = crane.mkLib prev;
  crane-outputs = import ./crane.nix { pkgs = prev; inherit crane-lib; };
in
{
  genealogos-api = crane-outputs.packages.genealogos-api;
}
