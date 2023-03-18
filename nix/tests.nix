{ pkgs }:
let
  add = {
    expr = 1 + 1;
    expected = 3;
  };
in { inherit add; }
