{ config, lib, pkgs, nixos-conf-editor, ... }:
with lib;
let
  cfg = config.programs.nixos-conf-editor;
  jsonFormat = pkgs.formats.json { };
in
{
  options = {
    programs.nixos-conf-editor = {
      enable = mkEnableOption (lib.mdDoc "nixos-conf-editor");
      systemconfig = mkOption {
        type = with types; nullOr str;
        default = null;
        example = literalExpression ''"/etc/nixos/configuration.nix"'';
        description = ''Where NixOS Configuration Editor looks for your system configuration.'';
      };
      flake = mkOption {
        type = with types; nullOr str;
        default = null;
        example = literalExpression ''"/etc/nixos/flake.nix"'';
        description = ''Where NixOS Configuration Editor looks for your system flake file.'';
      };
      flakearg = mkOption {
        type = with types; nullOr str;
        default = null;
        example = literalExpression ''user'';
        description = lib.mdDoc ''The flake argument to use when rebuilding the system. `nixos-rebuild switch --flake $\{programs.nixos-conf-editor.flake}#$\{programs.nixos-conf-editor.flakearg}`'';
      };
    };
  };

  config = mkMerge [
    (mkIf (cfg.enable || cfg.systemconfig != null || cfg.flake != null || cfg.flakearg != null) {
      environment.etc."nixos-conf-editor/config.json".source = jsonFormat.generate "config.json" cfg;
    })
    (mkIf (cfg.enable) {
      environment.systemPackages = [ nixos-conf-editor ];
    })
  ];
}
