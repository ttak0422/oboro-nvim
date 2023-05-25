{
  description = "Neovim configuration manager.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.flake-compat.follows = "flake-compat";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
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
    { self, nixpkgs, flake-utils, pre-commit-hooks, fenix, crane, ... }@inputs:
    flake-utils.lib.eachSystem [

      # TODO: support linux
      # "x86_64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays =
            [ fenix.overlays.default inputs.nix-filter.overlays.default ];
        };
        toolchain = pkgs.fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rustfmt"
          "rustc"
        ];
        craneLib = crane.lib.${system}.overrideToolchain toolchain;

        inherit (builtins) readFile;
        inherit (pkgs) nix-filter;
        inherit (pkgs.vimUtils) buildVimPlugin;
        inherit (pkgs.writers) writePython3Bin;
        inherit (pkgs.lib) optionals;
        inherit (pkgs.stdenv) isDarwin isx86_64;
        inherit (craneLib)
          path cleanCargoSource buildDepsOnly cargoClippy cargoNextest
          buildPackage;

        scripts = {
          preprocess = writePython3Bin "preprocess" { libraries = [ ]; }
            (readFile ./scripts/preprocess.py);
        };

        oboro = {
          resolver = rec {
            commonArgs = {
              src = cleanCargoSource (path ./.);
              buildInputs = [ ] ++ optionals isDarwin
                (with pkgs; [ libiconv darwin.apple_sdk.frameworks.Security ])
                ++ optionals (isDarwin && isx86_64)
                (with pkgs; [ darwin.apple_sdk.frameworks.CoreFoundation ]);
              nativeBuildgInputs = [ ];
              cargoToml = ./resolver/Cargo.toml;
              cargoVendorDir = null;
            };
            cargoArtifacts =
              buildDepsOnly (commonArgs // { pname = "oboro-resolver-deps"; });
            clippy = cargoClippy (commonArgs // { inherit cargoArtifacts; });
            nextest = cargoNextest (commonArgs // { inherit cargoArtifacts; });
            app = buildPackage (commonArgs // { inherit cargoArtifacts; });
          };
          # TODO: optimized , normal
          vimPlugin = buildVimPlugin {
            inherit (oboro) version;
            pname = "oboro-nvim";
            src = nix-filter {
              root = ./.;
              include = [ "lua" ];
            };
            buildPhase = ''
              ${scripts.preprocess}/bin/preprocess OPTIMIZED lua/oboro/init.lua
            '';
            preferLocalBuild = true;
          };
          version = "0.1.0";
          wip = scripts;
        };
      in {
        darwinModules = rec {
          oboro-nvim = import ./nix/module.nix { inherit oboro; };
          default = oboro-nvim;
        };

        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              deadnix.enable = true;
              stylua.enable = true;
              nixfmt.enable = true;
              statix.enable = true;
              rustfmt.enable = true;
            };
          };
          inherit (oboro.resolver) clippy nextest app;
          nixTest = import ./nix/checks { inherit pkgs; };
        };

        devShells.default = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          packages = [ toolchain ];
          inputsFrom = [ oboro ];
          RUST_BACKTRACE = "full";
        };
      });
}
