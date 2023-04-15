{ oboro }:
{ config, pkgs, lib, ... }:
let
  inherit (builtins) toJSON map;
  inherit (pkgs) writeText stdenv;
  inherit (lib) mkIf mkEnableOption flatten;
  inherit (lib.strings) concatStringsSep;
  inherit (lib.lists) unique;
  inherit (stdenv) mkDerivation;
  inherit (import ./types.nix { inherit pkgs lib; })
    nvimConfig oboroPluginConfig;
  inherit (import ./adapter.nix { inherit pkgs lib; })
    toStartPlugin toOptPlugin toBundle expandPlugin extractExtraPackages;

  cfg = config.programs.oboro-nvim;

  startPlugins = map toStartPlugin ([ oboro.vimPlugin ] ++ cfg.startPlugins);
  optPlugins = map toOptPlugin
    (flatten (map expandPlugin (cfg.optPlugins ++ cfg.bundles)));
  bundles = map toBundle cfg.bundles;

  extraPackages = flatten
    (map extractExtraPackages [ cfg.startPlugins cfg.optPlugins cfg.bundles ])
    ++ cfg.extraPackages;

  oboroJson = writeText "oboro.json"
    (toJSON { inherit startPlugins optPlugins bundles; });

  oboroStats = let
    startPackagesStr = concatStringsSep "\n" ([ "- start plugins" ]
      ++ (map (p: "  ${p.plugin}") (unique startPlugins)));
    optPackagesStr = concatStringsSep "\n"
      ([ "- opt plugins" ] ++ (map (p: "  ${p.plugin}") (unique optPlugins)));
  in writeText "stats.txt" ''
    - json (nix → rust)
      ${oboroJson}
    ${startPackagesStr}
    ${optPackagesStr}
  '';

  oboroSetupCode = let
    oboroRoot = mkDerivation {
      inherit (oboro) version;
      pname = "oboro-config-root";
      phases = [ "buildPhase" ];
      buildPhase = ''
        mkdir $out
        ${oboro.resolver}/bin/oboro-resolver ${oboroJson} $out
      '';
    };
  in ''
    require("oboro").setup({
      root = "${oboroRoot}",
      lazy_time = 100,
    })
  '';

in {
  options.programs.oboro-nvim = nvimConfig // oboroPluginConfig // {
    enable = mkEnableOption "oboro-nvim";
  };
  config = mkIf cfg.enable {

    programs.neovim = {
      inherit extraPackages;
      inherit (cfg) package withRuby withNodeJs withPython3;
      enable = true;
      plugins = (map (p: {
        inherit (p) plugin;
        optional = false;
      }) startPlugins) ++ (map (p: {
        inherit (p) plugin;
        optional = true;
      }) optPlugins);
    };
    xdg.configFile."nvim/init.lua".text = lib.mkAfter ''
      -- stats (${oboroStats})
      ${cfg.extraConfig}
      ${oboroSetupCode}
    '';
  };
}
