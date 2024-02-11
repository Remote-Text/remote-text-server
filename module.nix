{ config, pkgs, lib, ... }:

with lib;

let
  cfg = config.services.remote-text-server;
in
{
  options.services.remote-text-server = {
    enable = mkEnableOption "remote-text-server";
    package = mkOption {
      default = pkgs.callPackage ./. { texlive = pkgs.texliveFull; };
      defaultText = "remote-text-server";
      example = "inputs.remote-text-server.packages.${pkgs.system}.default.override { texlive = pkgs.texliveMinimal; }";
      description = "The remote-text-server package to use";
      type = types.package;
    };
    port = mkOption {
      type = types.port;
      default = 7870;
      example = 46264;
      description = "The port to listen on. Currently ignored and always uses 3030";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.remote-text-server = {
      description = "RemoteText Server";

      script = ''
        cd $STATE_DIRECTORY
        ${cfg.package}/bin/remote-text-server --port ${toString cfg.port}
      '';

      serviceConfig = {
        DynamicUser = true;
        # EnvironmentFile = "/etc/jekyll-comments-env";
        StateDirectory = "remote-text-server";

        PrivateDevices = true;
        PrivateMounts = true;
        PrivateUsers = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
      };

      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
    };
    # unnecessary bc tailscale is open. also should be set by the end user
    # networking.firewall.interfaces."tailscale0".allowedTCPPorts = [ cfg.port ];
  };
}
