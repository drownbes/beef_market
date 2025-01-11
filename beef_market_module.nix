{ config, lib, pkgs, ... }:
let
  cfg = config.services.beef_market;

  ollama = config.services.ollama;
  geckodriver = config.services.geckodriver;

  appConfig = pkgs.writeText "config.toml" ''
    [db]
    conn_str = "${cfg.stateDir}/db.sqlite"
    
    [ollama]
    host = "${ollama.host}"
    port = ${toString ollama.port}
    embedding_model = "${cfg.embeddingModel}"
    chat_model = "${cfg.chatModel}"
    
    [geckodriver]
    host = "${geckodriver.host}"
    port = ${toString geckodriver.port}
  '';
in
{
  imports = [
    ./geckodriver_module.nix
  ];

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

    chatModel = lib.mkOption {
      type = lib.types.str;
      default = "llama3.1:8b";
      description = ''
        The model for chat 
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
      default = cfg.user;
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

    services.ollama = {
      enable = true;
    };

    services.geckodriver = {
      enable = true;
    };

    users.users.${cfg.user} = {
      isSystemUser = true;
      createHome = true;
      group = cfg.group;
      home = "/var/lib/beef_market";
    };

    users.groups.${cfg.group} = {};

    systemd.services.beef_market = {
      description = "Tallinn Beef market price tracker";
      after = [ "geckodriver.service" "network.target"];
      requires = [ "geckodriver.service" ];
      wantedBy = [ "multi-user.target" ];
      environment.RUST_LOG = "info";
      serviceConfig = {
        ExecStart = "${pkgs.beef_market}/bin/beef_market ${appConfig}";
        Restart = "always";
        WorkingDir = "/var/lib/beef_market";
      };
    };
  };
}
