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
          targets = ["x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu"];
        };
      in
        with pkgs; {
          packages.default = callPackage ./nix/package.nix {inherit rust;};
          packages.windows = callPackage ./nix/package.nix {
            inherit rust;
            buildWindows = true;
          };

          devShells.default = mkShell {
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
            packages = [
              pkg-config
              rust
              lz4
              pkgs.pkgsCross.mingwW64.stdenv.cc
            ];
          };
        }
    )
    // {
      homeManagerModules.default = import ./nix/home-manager.nix self;
    };
}
