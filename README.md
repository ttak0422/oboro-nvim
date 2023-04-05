# oboro-nvim

Neovim configuration manager for Nix.

### Example

```nix
{
  programs.oboro-nvim = {
    enable = true;
    extraConfig = ''
      -- config lua
    '';
    extraPackages = with pkgs; [ delta ];

    startPlugins = with pkgs.vimPlugins; [
      vim-sensible
      {
        plugin = tokyonight-nvim;
        startup = ''
          vim.cmd[[colorscheme tokyonight]]
        '';
      }
      {
        plugin = persisted-nvim;
        startup = {
          lang = "lua";
          code = readFile ./persisted.lua;
          args = { exclude_ft_path = ./shared/exclude_ft.lua; };
        };
      }
      # ...
    ];

    optPlugins = with pkgs.vimPlugins; [
      {
        plugin = mkdir-nvim;
        events = [ "CmdlineEnter" ];
      }
      {
        plugin = rust-tools-nvim;
        depends = [ plenary-nvim nvim-dap ];
        dependBundles = [ "lsp" ];
        config = {
          lang = "lua";
          code = ''
            local rt = require("rust-tools")
            rt.setup({
              server = {
                on_attach = dofile(args.on_attach_path),
                capabilities = dofile(args.capabilities_path),
                settings = {
                  ["rust-analyzer"] = {
                    files = {
                      excludeDirs = { ".direnv" },
                    },
                  },
                },
              },
            })
            rt.inlay_hints.enable()

            vim.cmd([[LspStart]])
          ''
          args = {
            on_attach_path = ./shared/on_attach.lua;
            capabilities_path = ./shared/capabilities.lua;
          };
        };
        filetypes = [ "rust" ];
      }
      # ...
    ];

    bundles = with pkgs.vimPlugins; [
      {
        name = "lsp";
        plugins = [
          {
            plugin = nvim-lspconfig;
            extraPackages = with pkgs; [
              nil
              rust-analyzer
              nodePackages.typescript
              nodePackages.vscode-langservers-extracted
            ];
            config = {
              lang = "lua";
              code = readFile ./lspconfig.lua;
              args = {
                on_attach_path = ./shared/on_attach.lua;
                capabilities_path = ./shared/capabilities.lua;
                eslint_cmd = [
                  "${pkgs.nodePackages.vscode-langservers-extracted}/bin/vscode-eslint-language-server"
                  "--stdio"
                ];
                tsserver_cmd = [
                  "${pkgs.nodePackages.typescript-language-server}/bin/typescript-language-server"
                  "--stdio"
                ];
                tsserver_path =
                  "${pkgs.nodePackages.typescript}/lib/node_modules/typescript/lib/";
              };
            };
          }
          {
            plugin = actions-preview-nvim;
            config = readFile ./actions-preview.lua;
            dependBundles = [ "telescope" ];
            modules = [ "actions-preview" ];
          }
          # ...
        ];
        depends = [{
          plugin = fidget-nvim;
          config = readFile ./fidget.lua;
        }];
        lazy = true;
      }
      {
        name = "skk";
        plugins = [
          skkeleton
          {
            plugin = skkeleton_indicator-nvim;
            config = readFile ./skk-indicator.lua;
          }
        ];
        depends = [ denops-vim ];
        config = {
          lang = "vim";
          code = ''
            call skkeleton#config({
                  \ 'globalJisyo': s:args['jisyo_path'],
                  \ 'globalJisyoEncoding': 'utf-8',
                  \ 'useSkkServer': v:true,
                  \ 'skkServerHost': '127.0.0.1',
                  \ 'skkServerPort': 1178,
                  \ 'skkServerReqEnc': 'euc-jp',
                  \ 'skkServerResEnc': 'euc-jp',
                  \ 'markerHenkan': '',
                  \ 'markerHenkanSelect': '',
                  \ })
            imap <C-j> <Plug>(skkeleton-enable)
            cmap <C-j> <Plug>(skkeleton-enable)
          '';
          args = { jisyo_path = "${pkgs.skk-dicts}/share/SKK-JISYO.L"; };
        };
        lazy = true;
      }
      # ...
    ];
    withPython3 = true;
  };
}
```

### Plugin config (opt.programs.orobo-nvim)

| Property | Type | Default | Description |
|:-:|:-:|:-:|:-:|
| enable | boolean | false | enable oboro |
| package | package | pkgs.neovim-unwrapped | neovim package |
| extraPackages | package list | [] | nix packages |
| withNodeJs | boolean | false | enable node provider |
| withPython3 | boolean | false | enable python3 provider |
| withRuby | boolean | false | enable ruby provider |
| startPlugins | (package \| startPluginConfig) list | [] | `start` plugins |
| optPlugins | (package \| optPluginConfig) list | [] | `opt` plugins |
| bundles | bundleConfig list | [] | plugin bundles |

### Oboro Types

##### startPluginConfig

| Property | Type | Default | Description |
|:-:|:-:|:-:|:-:|
| plugin | package | - | nix vim plugin package |
| startup | string \| startupDetail | "" | configured on startup |
| extraPackages | package list | [] | nix packages |

##### startupDetail

| Property | Type | Default | Description |
|:-:|:-:|:-:|:-:|
| lang | "vim" \| "lua"  | - | language |
| code | string | "" | config code |
| args | attrs | {} | arguments |

##### optPluginConfig

| Property | Type | Default | Description |
|:-:|:-:|:-:|:-:|
| plugin | package | - | nix vim plugin package |
| startup | string \| startupDetail | "" | configured on startup |
| extraPackages | package list | [] | nix packages |
| preConfig | string \| configDetail | "" | configured before load plugin |
| config | string \| configDetail | "" | configured on load plugin |
| depends | (package \| optPluginConfig) list | [] | plugin dependencies |
| dependBundles | string list | [] | bundle dependsncies |
| modules | string list | [] | load plugin on required modules |
| events | string list | [] | load plugin on event triggered |
| filetypes | string list | [] | load plugin on load filetypes |
| commands | string list | [] | load plugin on execute commands |
| lazy | boolean | false | load plugin using timer |

##### bundleConfig

| Property | Type | Default | Description |
|:-:|:-:|:-:|:-:|
| name | string | - | bundle name |
| plugins | package list | [] | nix vim plugin packages |
| startup | string \| startupDetail | "" | configured on startup |
| extraPackages | package list | [] | nix packages |
| preConfig | string \| configDetail | "" | configured before load plugin |
| config | string \| configDetail | "" | configured on load plugin |
| depends | (package \| optPluginConfig) list | [] | plugin dependencies |
| dependBundles | string list | [] | bundle dependsncies |
| modules | string list | [] | load plugin on required modules |
| events | string list | [] | load plugin on event triggered |
| filetypes | string list | [] | load plugin on load filetypes |
| commands | string list | [] | load plugin on execute commands |
| lazy | boolean | false | load plugin using timer |

##### configDetail

| Property | Type | Default | Description |
|:-:|:-:|:-:|:-:|
| lang | "vim" \| "lua"  | - | language |
| code | string | "" | config code |
| args | attrs | {} | arguments |
