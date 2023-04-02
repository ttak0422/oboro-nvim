{
  description = "Neovim configuration manager.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-compat.follows = "";
      inputs.rust-overlay.follows = "";
    };
  };

  outputs =
    { self, nixpkgs, flake-utils, fenix, crane, ... }@inputs:
    flake-utils.lib.eachSystem [
      # TODO: support linux
      # "x86_64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ] (system:
      let
        VERSION = "0.0.1";

        pkgs = import nixpkgs {
          inherit system;
          overlays =
            [ fenix.overlays.default inputs.nix-filter.overlays.default ];
        };
        craneLib = crane.lib.${system};
        toolchain = pkgs.fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rustfmt"
          "rustc"
        ];

        inherit (builtins) readFile;
        inherit (pkgs) vimUtils nix-filter;
        inherit (pkgs.writers) writePython3Bin;
        inherit (pkgs.lib) optionals;
        inherit (pkgs.stdenv) isDarwin isx86_64;
        inherit (craneLib.overrideToolchain toolchain)
          buildDepsOnly buildPackage cargoClippy cargoFmt cargoNextest;

        scripts = {
          preprocess = writePython3Bin "preprocess" { libraries = [ ]; }
            (readFile ./scripts/preprocess.py);
        };

        oboro = rec {
          args = let
            args' = {
              src = craneLib.cleanCargoSource ./.;
              buildInputs = [ ] ++ optionals isDarwin
                (with pkgs; [ libiconv darwin.apple_sdk.frameworks.Security ])
                ++ optionals (isDarwin && isx86_64)
                (with pkgs; [ darwin.apple_sdk.frameworks.CoreFoundation ]);
              cargoToml = ./resolver/Cargo.toml;
              cargoVendorDir = null;
            };
          in args' // { cargoArtifacts = buildDepsOnly args'; };
          resolver = buildPackage args;
          # TODO: optimized , normal
          vimPlugin = vimUtils.buildVimPlugin {
            pname = "oboro-nvim";
            version = oboro.version;
            src = nix-filter {
              root = ./.;
              include = [ "lua" ];
            };
            buildPhase = ''
              ${scripts.preprocess}/bin/preprocess OPTIMIZED lua/oboro/init.lua
            '';
            preferLocalBuild = true;
          };
          version = VERSION;
          wip = scripts;
        };
      in {
        darwinModules = rec {
          oboro-nvim =
            import ./nix/module.nix { inherit oboro nix-filter scripts; };
          default = oboro-nvim;
        };

        checks = {
          clippy = cargoClippy oboro.args;
          fmt = cargoFmt oboro.args;
          rustTest = cargoNextest oboro.args;
        };

        devShells.default = pkgs.mkShell {
          packages = [ toolchain ];
          inputsFrom = [ oboro ];
          RUST_BACKTRACE = "full";
        };
      });
}
