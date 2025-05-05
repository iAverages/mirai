{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
          targets = ["arm-unknown-linux-gnueabihf"];
        };
      in
        with pkgs; {
          packages.default = callPackage ./nix/package.nix {};

          devShells.default = mkShell {
            packages = [
              pkg-config
              rust
              lz4
            ];
          };
        }
    )
    // {
      homeManagerModules.default = import ./nix/home-manager.nix self;
    };
}
