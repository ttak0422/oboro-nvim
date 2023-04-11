{ pkgs, lib }:
let
  fixture = import ./fixture.nix { };
  sut = import ./../adapter.nix { inherit pkgs lib; };
  inherit (builtins) elemAt;
  inherit (fixture) vimPluginPackages startPlugin optPlugin bundlePlugin;
in {
  test_simple_package_to_StartPlugin = {
    expr = sut.toStartPlugin (elemAt vimPluginPackages 0);
    expected = {
      id = "dummy1";
      plugin = { pname = "dummy1"; };
      startup = "";
    };
  };
  test_configured_start_plugin_to_StartPlugin = {
    expr = sut.toStartPlugin startPlugin.filled;
    expected = {
      id = "dummy1";
      plugin = { pname = "dummy1"; };
      startup = ''
        local args = vim.json.decode([[{"start":"start"}]])
        start startup
      '';
    };
  };

  test_simple_package_to_OptPlugin = {
    expr = sut.toOptPlugin (elemAt vimPluginPackages 0);
    expected = {
      id = "dummy1";
      plugin = { pname = "dummy1"; };
      startup = "";
      preConfig = "";
      config = "";
      deps = [ ];
      depBundles = [ ];
      mods = [ ];
      evs = [ ];
      fts = [ ];
      cmds = [ ];
      lazy = false;
    };
  };
  test_configured_opt_plugin_to_OptPlugin = {
    expr = sut.toOptPlugin optPlugin.filled;
    expected = {
      id = "dummy1";
      plugin = { pname = "dummy1"; };
      startup = "opt startup";
      preConfig = ''
        vim.cmd([[
          let s:args = json_decode('{"foo":"foo"}')
          opt preConfig
        ]])
      '';
      config = ''
        local args = vim.json.decode([[{"bar":1}]])
        opt config
      '';
      deps = [ "dummy2" "dummy3" ];
      depBundles = [ "bundle1" ];
      mods = [ "module" ];
      evs = [ "event" ];
      fts = [ "filetype" ];
      cmds = [ "command" ];
      lazy = true;
    };
  };

  test_simple_bundle_to_Bundle = {
    expr = sut.toBundle "dummy";
    expected = {
      id = "dummy";
      startup = "";
      preConfig = "";
      config = "";
      plugins = [ ];
      deps = [ ];
      depBundles = [ ];
      mods = [ ];
      evs = [ ];
      fts = [ ];
      cmds = [ ];
      lazy = false;
    };
  };
  test_configured_bundle_to_Bundle = {
    expr = sut.toBundle bundlePlugin.filled;
    expected = {
      id = "dummy";
      startup = "bundle startup";
      preConfig = "bundle preConfig";
      config = "bundle config";
      plugins = [ "dummy1" "dummy2" ];
      deps = [ "dummy3" "dummy4" ];
      depBundles = [ "bundle_depend_bundle" ];
      mods = [ "bundle_module" ];
      evs = [ "bundle_event" ];
      fts = [ "bundle_filetype" ];
      cmds = [ "bundle_command" ];
      lazy = false;
    };
  };
}
