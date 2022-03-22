{ channel ? "stable"
, ... }:

let
  moz_overlay = import (
    builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz
  );

  pkgs = import <nixpkgs-unstable> { overlays = [ moz_overlay ]; };

  rustPackages = (with (pkgs.rustChannelOf { inherit channel; }); [
    rust-std
    rust-src
    (rust.override { extensions = ["rust-src" ]; })
    rustc
    cargo
  ]);
in
pkgs.mkShell {
  buildInputs = rustPackages
    ++ (with pkgs; [
      atk
      cairo
      gdk-pixbuf
      gtk3
      openssl
      pango
      pkg-config
      webkitgtk
    ]);

  shellHook = ''
    export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/src"

    echo " rustc: ''$(rustc --version)"
    echo " cargo: ''$(cargo --version)"
    echo "... have fun!"
  '';

  LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
}


