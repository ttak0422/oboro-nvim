# Adapt nix configuration to formatted json (src/config/input.rs).
{ pkgs, lib }:
let
  inherit (lib) flatten;
  inherit (builtins) map toJSON;
  inherit (import ./types.nix { inherit pkgs lib; })
    startPluginConfigDefault optPluginConfigDefault bundleConfigDefault;

  # extract id.
  #
  # Type:
  # --------------------
  # (package | optPluginConfig | bundleConfig) -> str;
  extractId = x:
    if x ? type' && x.type' == "plugin" then
      x.plugin.pname
    else if x ? type' && x.type' == "bundle" then
      x.name
    else
      x.pname;

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
    else if x ? lang && x.lang == "vim" then
      if x.args != { } then ''
        vim.cmd([[
          ${x.code}
        ]])
      '' else ''
        vim.cmd([[
          let s:args = json_decode('${toJSON x.args}')
          ${x.code}
        ]])
      ''
    else
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
    else if x ? lang && x.lang == "vim" then
      if x.args != { } then ''
        vim.cmd([[
          let s:args = json_decode('${toJSON x.args}')
          ${x.code}
        ]])
      '' else ''
        vim.cmd([[
          ${x.code}
        ]])
      ''
    else
      x;
in rec {
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
      preConfig = mkConfigCode plugin.preConfig;
      config = mkConfigCode plugin.config;
      deps = map extractId plugin.depends;
      depBundles = plugin.dependBundles;
      mods = plugin.modules;
      evs = plugin.events;
      fts = plugin.filetypes;
      cmds = plugin.commands;
    } else
      let default = optPluginConfigDefault;
      in {
        inherit plugin;
        inherit (default) startup preConfig config lazy;
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
      preConfig = mkConfigCode bundle.preConfig;
      config = mkConfigCode bundle.config;
      plugins = map extractId bundle.plugins;
      deps = map extractId bundle.depends;
      depBundles = bundle.dependBundles;
      mods = bundle.modules;
      evs = bundle.events;
      fts = bundle.filetypes;
      cmds = bundle.commands;
    } else
      let default = bundleConfigDefault;
      in {
        inherit (default) startup preConfig config lazy;
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
  # (package | optPluginConfig | bundleConfig) -> List[package | optPluginConfig]
  expandPlugin = x:
    if x ? type' && x.type' == "plugin" then
      let depends = if x ? depends then x.depends else [ ];
      in flatten ([ x ] ++ (map expandPlugin depends))
    else if x ? type' && x.type' == "bundle" then
      let depends = if x ? depends then x.depends else [ ];
      in flatten (map expandPlugin (x.plugins ++ depends))
    else
      [ x ];

  # extract extraPackages.
  #
  # Type:
  # --------------------
  # (package | optPluginConfig | bundleConfig) -> List[extraPackage]
  extractExtraPackages = x:
    flatten ((map (y: if y ? extraPackages then y.extraPackages else [ ]))
      (flatten (map expandPlugin x)));
}
