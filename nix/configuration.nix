{ ... }:

let
  port = 9000;
in
{
  virtualisation.vmVariant = {
    virtualisation = {
      memorySize = 2048; # Use 2048MiB memory.
      cores = 3;
      forwardPorts = [
        {
          from = "host";
          guest.port = port;
          host.port = port;
        }
      ];
    };
  };

  users.users.alice = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    password = "genealogos";
  };

  services.genealogos = {
    enable = true;
    rocketConfig = {
      release = {
        port = port;
        address = "0.0.0.0";
      };
    };
  };
}

