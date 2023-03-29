# Adapt nix configuration to formatted json (src/config/input.rs).
{ pkgs, lib }:
let
  inherit (builtins) map toJSON;
  inherit (import ./types.nix { inherit pkgs lib; })
    startPluginConfigDefault optPluginConfigDefault bundleConfigDefault;

  # make config.
  #
  # Type:
  # --------------------
  # (lines | configDetail) -> lines;
  mkStartupCode = x:
    if x ? lang && x.lang == "lua" then
      if x.args != { } then ''
        local args = vim.json.decode([[${toJSON x.args}]])
        ${x.code}
      '' else
        x.code
    else if x ? lang && x.lang == "vim" then ''
      vim.cmd([[
        ${x.code}
      ]])
    '' else
      x;

  # make config.
  #
  # Type:
  # --------------------
  # (lines | configDetail) -> lines;
  mkConfigCode = x:
    if x ? lang && x.lang == "lua" then
      if x.args != { } then ''
        local args = vim.json.decode([[${toJSON x.args}]])
        ${x.code}
      '' else
        x.code
    else if x ? lang && x.lang == "vim" then ''
      vim.cmd([[
        ${x.code}
      ]])
    '' else
      x;
in {
  # adapt to `StartPlugin`.
  #
  # Type:
  # --------------------
  # (package | startPluginConfig) -> StartPlugin (src/config/input.rs)
  toStartPlugin = plugin:
    if plugin ? plugin then {
      inherit (plugin) plugin;
      id = plugin.plugin.pname;
      startup = mkStartupCode plugin.startup;
    } else {
      inherit plugin;
      inherit (startPluginConfigDefault) startup;
      id = plugin.pname;
    };

  # adapt to `OptPlugin`.
  #
  # Type:
  # --------------------
  # (package | optPluginConfig) -> OptPlugin (src/config/input.rs)
  toOptPlugin = plugin:
    if plugin ? plugin then {
      inherit (plugin) plugin lazy;
      id = plugin.plugin.pname;
      startup = mkStartupCode plugin.startup;
      config = mkConfigCode plugin.config;
      deps = map (p: p.pname) plugin.depends;
      depBundles = plugin.dependBundles;
      mods = plugin.modules;
      evs = plugin.events;
      fts = plugin.filetypes;
      cmds = plugin.commands;
    } else
      let default = optPluginConfigDefault;
      in {
        inherit plugin;
        inherit (default) startup config lazy;
        id = plugin.pname;
        deps = default.depends;
        depBundles = default.dependBundles;
        mods = default.modules;
        evs = default.events;
        fts = default.filetypes;
        cmds = default.commands;
      };

  # adapt to `Bundle`.
  #
  # Type:
  # --------------------
  # (str | BundleConfig) -> Bundle (src/config/input.rs)
  toBundle = bundle:
    if bundle ? name then {
      inherit (bundle) lazy;
      id = bundle.name;
      startup = mkStartupCode bundle.startup;
      config = mkConfigCode bundle.config;
      plugins = map (p: p.pname) bundle.plugins;
      deps = map (p: p.pname) bundle.depends;
      depBundles = bundle.dependBundles;
      mods = bundle.modules;
      evs = bundle.events;
      fts = bundle.filetypes;
      cmds = bundle.commands;
    } else
      let default = bundleConfigDefault;
      in {
        inherit (default) start config lazy;
        id = bundle;
        plugins = default.plugins;
        deps = default.depends;
        depBundles = default.dependBundles;
        mods = default.modules;
        evs = default.events;
        fts = default.filetypes;
        cmds = default.commands;
      };

  # expand packages.
  #
  # Type:
  # --------------------
  # (package | optPluginConfig | bundleConfig) -> with types; listOf (either package optPluginConfig)
  expandPlugin = x:
    if x ? type' && x.type' == "plugin" then
      [ x ] ++ x.depends
    else if x ? type' && x.type' == "bundle" then
      x.plugins ++ x.depends
    else
      [ x ];
}
