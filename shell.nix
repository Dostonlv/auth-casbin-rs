flake:
{ pkgs, ... }:
let
  # Hostplatform system
  system = pkgs.stdenv.hostPlatform.system;
in
pkgs.mkShell {
  packages = with pkgs; [
    nixd
    statix
    deadnix
    alejandra

    rustfmt
    clippy
    rust-analyzer
    cargo-watch

    # Other packages here
    openssl
    libressl
    curl
    # ...
    stdenv 
    gcc
    rustc
    sqlx-cli
    redis
    sqlite
    rlwrap
  ];

  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  shellHook = ''
    # Extra steps to do while activating development shell
  '';
}
