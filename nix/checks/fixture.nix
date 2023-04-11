{ ... }:
let inherit (builtins) elemAt;
in rec {
  extraPackages = [{ pname = "extra1"; }];
  vimPluginPackages = [
    { pname = "dummy1"; }
    { pname = "dummy2"; }
    { pname = "dummy3"; }
    { pname = "dummy4"; }
    { pname = "dummy5"; }
  ];

  startPlugin = {
    filled = {
      type' = "plugin";
      plugin = elemAt vimPluginPackages 0;
      startup = {
        lang = "lua";
        code = "start startup";
        args = { start = "start"; };
      };
      extraPackages = extraPackages;
    };
  };

  optPlugin = {
    filled = {
      type' = "plugin";
      plugin = elemAt vimPluginPackages 0;
      startup = "opt startup";
      extraPackages = extraPackages;
      preConfig = {
        lang = "vim";
        code = "opt preConfig";
        args = { foo = "foo"; };
      };
      config = {
        lang = "lua";
        code = "opt config";
        args = { bar = 1; };
      };
      depends = [
        (elemAt vimPluginPackages 1)
        {
          type' = "plugin";
          plugin = elemAt vimPluginPackages 2;
          startup = "opt startup nested";
          extraPackages = [{ pname = "extra2"; }];
          preConfig = "opt preConfig nested";
          config = "opt config nested";
          depends = [ elemAt vimPluginPackages 3 ];
          dependBundles = [ "bundle_nested" ];
          modules = [ "module_nested" ];
          events = [ "event_nested" ];
          filetypes = [ "filetype_nested" ];
          commands = [ "command_nested" ];
          lazy = false;
        }
      ];
      dependBundles = [ "bundle1" ];
      modules = [ "module" ];
      events = [ "event" ];
      filetypes = [ "filetype" ];
      commands = [ "command" ];
      lazy = true;
    };
  };

  bundlePlugin = {
    filled = {
      type' = "bundle";
      name = "dummy";
      plugins = [
        (elemAt vimPluginPackages 0)
        {
          type' = "plugin";
          plugin = elemAt vimPluginPackages 1;
          startup = "bundle plugin nested startup";
          extraPackages = [{ pname = "extra2"; }];
          preConfig = "bundle plugin nested preConfig";
          config = "bundle plugin nested config";
          depends = [ elemAt vimPluginPackages 3 ];
          dependBundles = [ "bundle_plugin_nested_bundle" ];
          modules = [ "bundle_plugin_nested_module" ];
          events = [ "bundle_plugin_nested_event" ];
          filetypes = [ "bundle_plugin_nested_filetype" ];
          commands = [ "bundle_plugin_nested_command" ];
          lazy = false;
        }
      ];
      startup = "bundle startup";
      extraPackages = extraPackages;
      preConfig = "bundle preConfig";
      config = "bundle config";
      depends = [
        (elemAt vimPluginPackages 2)
        {
          type' = "plugin";
          plugin = elemAt vimPluginPackages 3;
          startup = "bundle depends nested startup";
          extraPackages = [{ pname = "extra3"; }];
          preConfig = "bundle depends nested preConfig";
          config = "bundle depends nested config";
          depends = [ elemAt vimPluginPackages 4 ];
          dependBundles = [ "bundle_depends_nested_bundle" ];
          modules = [ "bundle_depends_nested_module" ];
          events = [ "bundle_depends_nested_event" ];
          filetypes = [ "bundle_depends_nested_filetype" ];
          commands = [ "bundle_depends_nested_command" ];
          lazy = true;
        }
      ];
      dependBundles = [ "bundle_depend_bundle" ];
      modules = [ "bundle_module" ];
      events = [ "bundle_event" ];
      filetypes = [ "bundle_filetype" ];
      commands = [ "bundle_command" ];
      lazy = false;
    };
  };
}
