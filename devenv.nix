{ pkgs, lib, nixGL, ... }:

{
  packages = lib.optionals pkgs.stdenv.isDarwin
    (with pkgs.darwin.apple_sdk; [
      frameworks.Security
    ]) ++ lib.optionals pkgs.stdenv.isLinux ([
    pkgs.nixgl.auto.nixGLDefault
  ]) ++
  [ pkgs.rnix-lsp pkgs.nixpkgs-fmt ];

  languages.nix.enable = true;
  languages.rust = {
    enable = true;
    channel = "stable";
  };


  pre-commit.hooks = {
    shellcheck.enable = true;
    clippy.enable = true;
    rustfmt.enable = true;
  };
}
