self: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) types;
  cfg = config.services.mirai;
  tomlFormat = pkgs.formats.toml {};
in {
  options.services.mirai = {
    enable = lib.mkEnableOption "Enable the mirai service";
    settings = {
      log_level = lib.mkOption {
        type = types.enum ["trace" "debug" "info" "warn" "error"];
        default = "info";
      };

      content_manager_type = lib.mkOption {
        type = types.enum ["local" "git"];
        default = "local";
      };

      update_interval = lib.mkOption {
        type = types.int;
        default = 1440;
        description = "how often wallpaper will be updated";
      };

      local = {
        path = lib.mkOption {
          type = types.str;
          default = "";
          description = "local path for wallpapers";
          example = "~/Documents/wallpapers";
        };
      };

      git = {
        url = lib.mkOption {
          type = types.str;
          default = "";
          description = "git repository URL for content.";
          example = "https://github.com/user/repo.git";
        };

        path = lib.mkOption {
          type = types.str;
          default = "";
          description = "subdirectory within the git repository to use as content root.";
          example = "wallpapers";
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.settings.content_manager_type != "local" || cfg.settings.local.path != "";
        message = "services.mirai.settings.local.location must be set when content_manager_type is 'local'.";
      }
      {
        assertion = cfg.settings.content_manager_type != "git" || cfg.settings.git.url != "";
        message = "services.mirai.settings.git.url must be set when content_manager_type is 'git'.";
      }
    ];

    home = {
      packages = [self.packages.${pkgs.system}.default];

      file.".config/mirai/mirai.toml".source = tomlFormat.generate "mirai.toml" {
        log_level = cfg.settings.log_level;
        content_manager_type = cfg.settings.content_manager_type;
        update_interval = cfg.settings.update_interval;

        local = {
          path = cfg.settings.local.path;
        };

        git = {
          url = cfg.settings.git.url;
          path = cfg.settings.git.path;
        };
      };
    };

    systemd.user.services.mirai = {
      Unit = {
        Description = "Mirai background service";
        After = ["graphical-session.target"];
      };
      Service = {
        ExecStart = "${self.packages.${pkgs.system}.default}/bin/mirai";
        Restart = "always";
      };
      Install = {
        WantedBy = ["default.target"];
      };
    };
  };
}
