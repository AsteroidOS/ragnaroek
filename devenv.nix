{ pkgs, lib, ... }:

{
  packages = lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
    frameworks.Security
  ]);

  languages.nix.enable = true;
  languages.rust = {
    enable = true;
    version = "stable";
  };


  pre-commit.hooks = {
    shellcheck.enable = true;
    clippy.enable = true;
    rustfmt.enable = true;
  };
}
