
{ config, lib, pkgs, ... }:
let
  cfg = config.services.geckodriver;
in {
  options.services.geckodriver = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Enabled geckodriver background run
      '';
    };
    user = lib.mkOption {
      type = lib.types.str;
      default = "geckodriver";
      description = ''
        The system user under which geckodriver will run.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.geckodriver = {
      isSystemUser = true;
      group = cfg.user;
      createHome = true;
      home = "/var/lib/geckodriver";
      packages = [
        pkgs.firefox
      ];
    };
    users.groups.${cfg.user} = { };

    systemd.services.geckodriver = {
      description = "Geckodriver for scraping and automating";
      wantedBy = [ "multi-user.target" ];
      after = [
        "network.target"
      ];
      serviceConfig = {
        ExecStart = "${pkgs.geckodriver}/bin/geckodriver";
        Restart = "always";
        User = cfg.user;
        Group = cfg.user;
        StateDirectory = "/var/lib/geckodriver";
        WorkingDirectory = "/var/lib/geckodriver";
      };
    };
  };
}
