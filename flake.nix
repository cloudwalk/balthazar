{
  description = "Cloudwalk's Swiss Knife Tooling for Rust";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in rec {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs;
            [
              cargo-audit
              cargo-bloat
              crate2nix
              nixfmt
              openssl
              pkg-config
              pgcli
              postgresql
              rust-analyzer
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" "clippy" "rustfmt" ];
              })
            ] ++ lib.optionals pkgs.stdenv.isLinux [ cargo-watch ]
            ++ lib.optionals pkgs.stdenv.isDarwin
            (with pkgs.darwin.apple_sdk.frameworks; [
              nixpkgs.legacyPackages.x86_64-darwin.cargo-watch
              Security
              CoreServices
              CoreFoundation
              Foundation
              AppKit
            ]);
        };
      });
}
