{ ... }:

{
  virtualisation.vmVariant = {
    virtualisation = {
      memorySize = 2048; # Use 2048MiB memory.
      cores = 3;
      forwardPorts = [
        {
          from = "host";
          guest.port = 8000;
          host.port = 8000;
        }
      ];
    };
  };

  users.users.alice = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    password = "genealogos";
  };

  services.genealogos.enable = true;
}

