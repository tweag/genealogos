{ crane-lib }:
final: prev:
let
  crane-outputs = import ./crane.nix { pkgs = prev; inherit crane-lib; };
in
{
  genealogos-api = crane-outputs.packages.genealogos-api;
}
