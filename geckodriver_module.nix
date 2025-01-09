
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

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      example = "[::]";
      description = ''
          The host address which the geckodriver server HTTP interface listens to.
      '';
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 4444;
      example = 4444;
      description = ''
          Which port the geckodriver server listens to.
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
        ExecStart = "${pkgs.geckodriver}/bin/geckodriver --host ${cfg.host} --port ${toString cfg.port}";
        Restart = "always";
        User = cfg.user;
        Group = cfg.user;
        StateDirectory = "/var/lib/geckodriver";
        WorkingDirectory = "/var/lib/geckodriver";
      };
    };
  };
}
