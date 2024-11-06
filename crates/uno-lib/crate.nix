{ flake
, rust-project
, pkgs
, lib
, ...
}:

{
  autoWire = [ "doc" "clippy" ];
}
