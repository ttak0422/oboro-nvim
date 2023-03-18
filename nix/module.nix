{ nix-filter, oboro, scripts }:
{ config, pkgs, lib, ... }:
let
  inherit (builtins) toJSON;
  inherit (pkgs) writeText stdenv;
  inherit (lib) mkIf mkEnableOption flatten;
  inherit (stdenv) mkDerivation;
  inherit (import ./types.nix { inherit pkgs lib; })
    nvimConfig oboroPluginConfig;
  inherit (import ./adapter.nix { inherit pkgs lib; })
    toStartPlugin toOptPlugin toBundle expandPlugin;

  # scripts = callPackage ./../scripts { };

  cfg = config.programs.oboro-nvim;
  # oboroNvim = vimUtils.buildVimPlugin {
  #   pname = "oboro-nvim";
  #   version = oboro.version;
  #   src = nix-filter {
  #     root = ./..;
  #     include = [ "lua" ];
  #   };
  #   buildInputs = [ scripts.preprocess ];
  #   preferLocalBuild = true;
  # };
  startPlugins = map toStartPlugin ([ oboro.vimPlugin ] ++ cfg.startPlugins);
  optPlugins = map toOptPlugin
    (flatten (map expandPlugin (cfg.optPlugins ++ cfg.bundles)));
  bundles = map toBundle cfg.bundles;
  # startPlugins = map toStartPlugin ([ oboroNvim ] ++ cfg.startPlugins);

  # optPlugins = map toOptPlugin [ ];
  oboroSetupCode = let
    configRoot = { inherit startPlugins optPlugins bundles; };
    oboroJson = writeText "oboro.json" (toJSON configRoot);
    oboroRoot = mkDerivation {
      pname = "oboro-config-root";
      version = oboro.version;
      phases = [ "buildPhase" ];
      # buildInputs = [ oboro.resolver ];
      buildPhase = ''
        mkdir $out
        ${oboro.resolver}/bin/oboro-resolver ${oboroJson} $out
      '';
    };
  in ''
    -- JSON
    -- ${oboroJson}
    -- ${oboroRoot}
    -- ${oboro.vimPlugin}
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
      inherit (cfg) package withRuby withNodeJs withPython3;
      enable = true;
      plugins = 
      (map (p: { inherit (p) plugin; optional = false; }) startPlugins)
      ++ (map (p: { inherit (p) plugin; optional = true; }) optPlugins);
    };
    xdg.configFile."nvim/init.lua".text = lib.mkAfter ''
      ${oboroSetupCode}
    '';
  };
}
