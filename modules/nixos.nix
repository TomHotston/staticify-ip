inputs:
{ lib, pkgs, config, ... }:
let
  cfg = config.services.staticify-ip;
in with lib;{
  options = {
    services.staticify-ip = {
      enable = mkEnableOption "Enables staticify-ip service";

      token = mkOption {
        type = lib.types.str;
        example = true;
        description = "The API token used to configure the Cloudflare DNS";
      };

      configurationWebsite = mkOption {
        type = lib.types.str;
        example = true;
        description = "The website that the configuration is targetting";
      };

      zoneId = mkOption {
        type = lib.types.str;
        example = true;
        description = "The zone that is DNS configuration resides in";
      };

      runPeriod = mkOption {
        type = lib.types.str;
        example = true;
        description = "The period of which to run the timer";
      };
    };
  };

  config = let
    staticify-ip-pkg = inputs.self.packages.${pkgs.stdenv.system}.default;
  in lib.mkIf cfg.enable
  {
    # Install staticify-ip
    environment.systemPackages = [
      staticify-ip-pkg
    ];

    # configure service
    systemd.services."staticify-ip" = {
        script = ''
          ${staticify-ip-pkg}/bin/staticify-ip -c /etc/staticify-ip/staticify-ip.toml
        '';
        serviceConfig = {
        Type = "oneshot";
      };
    };

    # write config file
    environment.etc = {
      "staticify-ip/staticify-ip.toml" = {
        text = ''token = "${cfg.token}"
website = "${cfg.configurationWebsite}"
zone_id = "${cfg.zoneId}"'';
        mode = "0440";
      };
    };

    # Configure timer
    systemd.timers."staticify-ip" = {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = cfg.runPeriod;
        OnUnitActiveSec = cfg.runPeriod;
        Unit = "staticify-ip.service";
      };
    };
  };
}
