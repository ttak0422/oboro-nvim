{ pkgs, lib }:

let inherit (lib) types mkOption mkEnableOption literalExample;
in rec {
  defaults = {
    commonConfig = {
      startup = "";
      extraPackages = [ ];
    };
    lazyCoreConfig = {
      config = "";
      depends = [ ];
      dependBundles = [ ];
      modules = [ ];
      events = [ ];
      filetypes = [ ];
      commands = [ ];
      lazy = false;
    };
    pluginConfig = { type' = "plugin"; };
    bundleConfig = {
      type' = "bundle";
      plugins = [ ];
    };
  };

  options = let default = defaults.commonConfig;
  in rec {
    commonConfig = {
      type' = mkOption {
        type = types.enum [ "plugin" "bundle" ];
        description = "automatically set by oboro";
        visible = false;
      };
      startup = mkOption {
        type = with types; either lines startupDetail;
        default = default.startup;
      };
      extraPackages = mkOption {
        type = with types; listOf package;
        description = "nix packages.";
        default = default.extraPackages;
      };
    };
    lazyCoreConfig = let default = defaults.lazyCoreConfig;
    in {
      config = mkOption {
        type = with types; either lines configDetail;
        default = default.config;
      };
      depends = mkOption {
        type = with types; listOf package;
        default = default.depends;
      };
      dependBundles = mkOption {
        type = with types; listOf str;
        default = default.dependBundles;
      };
      modules = mkOption {
        type = with types; listOf str;
        default = default.modules;
      };
      events = mkOption {
        type = with types; listOf str;
        default = default.events;
      };
      filetypes = mkOption {
        type = with types; listOf str;
        default = default.filetypes;
      };
      commands = mkOption {
        type = with types; listOf str;
        default = default.commands;
      };
      lazy = mkEnableOption "lazy" // { default = default.lazy; };
    };
    pluginConfig = let default = defaults.pluginConfig;
    in {
      type' = commonConfig.type' // { default = default.type'; };
      plugin = mkOption {
        type = types.package;
        description = "vim plugin package.";
      };
    };
    bundleConfig = let default = defaults.bundleConfig;
    in {
      type' = commonConfig.type' // { default = default.type'; };
      name = mkOption {
        type = types.str;
        description = "bundle name";
      };
      plugins = mkOption {
        type = with types; listOf package;
        default = default.plugins;
      };
    };
  };

  # `start` plugin default config.
  startPluginConfigDefault = defaults.commonConfig // defaults.pluginConfig;

  # `opt` plugin default config.
  optPluginConfigDefault = defaults.commonConfig // defaults.lazyCoreConfig
    // defaults.pluginConfig;

  # bundle default config.
  bundleConfigDefault = defaults.commonConfig // defaults.lazyCoreConfig
    // defaults.bundleConfig;

  startupDetail = types.submodule {
    options = {
      lang = mkOption { type = types.enum [ "vim" "lua" ]; };
      code = mkOption {
        type = types.lines;
        default = "";
      };
      args = mkOption {
        type = types.attrs;
        default = { };
      };
    };
  };

  configDetail = types.submodule {
    options = {
      lang = mkOption { type = types.enum [ "vim" "lua" ]; };
      code = mkOption {
        type = types.lines;
        default = "";
      };
      args = mkOption {
        type = types.attrs;
        default = { };
      };
    };
  };

  # `start` plugin config.
  startPluginConfig =
    types.submodule { options = options.commonConfig // options.pluginConfig; };

  # `opt` plugin config.
  optPluginConfig = types.submodule {
    options = options.commonConfig // options.lazyCoreConfig
      // options.pluginConfig;
  };

  # bundle config.
  bundleConfig = types.submodule {
    options = options.commonConfig // options.lazyCoreConfig
      // options.bundleConfig;
  };

  # neovin alias config.
  nvimConfig = {
    package = mkOption {
      type = types.package;
      description = "alias for neovim.package";
      default = pkgs.neovim-unwrapped;
    };

    extraPackages = mkOption {
      type = with types; listOf package;
      description = "extraPackages";
      default = [ ];
    };

    extraConfig = mkOption {
      type = types.lines;
      description = "neovim configs in lua.";
      default = "";
      example = literalExample ''
        vim.wo.number=true
      '';
    };

    withNodeJs = mkEnableOption "withNodeJs" // {
      description = "alias for neovim.withNodeJs";
    };

    withPython3 = mkEnableOption "withPython3" // {
      description = "alias for neovim.withPython3";
    };

    withRuby = mkEnableOption "withRuby" // {
      description = "alias for neovim.withRuby";
    };
  };

  oboroPluginConfig = {
    startPlugins = mkOption {
      type = with types; listOf (either package startPluginConfig);
      description = "`start` plugins.";
      default = [ ];
    };

    optPlugins = mkOption {
      type = with types; listOf (either package optPluginConfig);
      description = "`opt` plugins.";
      default = [ ];
    };

    bundles = mkOption {
      type = types.listOf bundleConfig;
      description = "plugin bundles.";
      default = [ ];
    };
  };
}
