{ pkgs }:
let
  inherit (builtins) toJSON;
  inherit (pkgs) lib runCommand writeText;
  results =
    lib.runTests ({ } // (import ./adapter.spec.nix { inherit pkgs lib; }));
  resultsFile = writeText "errors.json" (toJSON results);
in runCommand "nix-ut" { } ''
  mkdir $out
  ${if results != [ ] then ''
    echo "failed nix test. see ${resultsFile}"
    exit 1
  '' else ''
    echo "all tests passed!"
    exit 0
  ''}
''
