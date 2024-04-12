{ genealogos-api }:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.genealogos;
in
{
  options = {
    services.genealogos = {
      enable = mkEnableOption
        (mdDoc "Genealogos, a Nix sbom generator");

      package = mkOption {
        type = types.package;
        default = genealogos-api;
        description = mdDoc ''
          The genealogos-api package to use.
        '';
      };
    };
  };

  config = mkIf (cfg.enable) {
    systemd.services.genealogos =
      {
        description = "Genealogos sbom generator";
        wantedBy = [ "multi-user.target" ];

        serviceConfig.ExecStart = "${cfg.package}/bin/genealogos-api";
      };
  };
}
