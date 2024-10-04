{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
    inputs.process-compose-flake.flakeModule
    inputs.cargo-doc-live.flakeModule
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    cargo-doc-live.crateName = "uno";

    rust-project = {
      # See /crates/*/crate.nix for the crate-specific Nix configuration
      crateNixFile = "crate.nix";
    };
  };
}
