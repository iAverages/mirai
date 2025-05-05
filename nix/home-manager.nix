self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.mirai;
in {
  options.services.mirai = {
    enable = lib.mkEnableOption "Enable the mirai service";
  };

  config = lib.mkIf cfg.enable {
    home.packages = [self.packages.${pkgs.system}.default];
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
