{ config, lib, options, pkgs, ... }:

with lib;

let
  cfg = config.services.genealogos;
in
{
  options = {
    services.genealogos = {
      enable = mkEnableOption
        (mdDoc "Genealogos, a Nix sbom generator");

      package = mkPackageOption pkgs "genealogos-api" { };
    };
  };

  config = mkIf (cfg.enable) {
    systemd.services.genealogos =
      {
        description = "Genealogos sbom generator";
        path = [ cfg.package ];
        wantedBy = [ "multi-user.target" ];

        script = ''
          ${cfg.package}/bin/genealogos-api
        '';
      };
  };
}
