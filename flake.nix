{
  description = "Rusic development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
            dioxus-cli
          ];

          buildInputs = with pkgs; [
            openssl
            glib
            gtk3
            libsoup_3
            webkitgtk_4_1
            xdotool
            
            nodejs
          ];

          LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
            openssl
            glib
            gtk3
            libsoup_3
            webkitgtk_4_1
            xdotool
          ];
        };
      }
    );
}
