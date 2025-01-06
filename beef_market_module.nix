{ config, lib, pkgs, ... }:
let
  cfg = config.services.beef_market;

  ollama = config.services.ollama;

  appConfig = pkgs.writeFile "config.toml" ''
    [db]
    conn_str = "${cfg.stateDir}/db.sqlite"
    
    [ollama]
    host = "${ollama.host}"
    port = ${ollama.port}
    embedding_model = "${cfg.embeddingModel}
    
    [geckodriver]
    host = "127.0.0.1"
    port = 4444
  '';
in
{
  options.services.beef_market = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Enables example module with two systemd services (serviceA and serviceB).
      '';
    };

    embeddingModel = lib.mkOption {
      type = lib.types.str;
      default = "snowflake-arctic-embed2";
      description = ''
        The model use for embeddings
      '';
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "beef_market";
      description = ''
        The system user under which beef_market will run.
      '';
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "beef_market";
      description = ''
        The system group under which beef_market will run.
      '';
    };
    stateDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/beef_market";
      description = ''
        The directory where beef_market stores its state (e.g., SQLite database).
      '';
    };

  };

  config = lib.mkIf cfg.enable {
    assertions =
      [ { assertion = ollama.enable;
          message = "Ollama service should be enabled";
        }
      ];

    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
    };

    users.groups.${cfg.group} = {};

    systemd.services.geckodriver = {
      description = "Geckodriver for scraping and automating";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.geckodriver}/bin/geckodriver";
        Restart = "always";
        ProtectSystem = "strict";
        ProtectHome = "yes";
      };
    };

    systemd.services.beef_market = {
      description = "Tallinn Beef market price tracker";
      after = [ "geckodriver.service" ];
      requires = [ "geckodriver.service" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.beef_market}/bin/beef_market";
        Restart = "always";
        WorkingDir = "/var/lib/beef_market";
        ProtectSystem = "strict";
        ProtectHome = "yes";
      };
    };
  };
}
