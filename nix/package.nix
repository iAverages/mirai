{
  rustPlatform,
  lib,
  pkgs,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage {
    pname = "mirai";
    inherit (cargoToml.package) version;

    cargoLock.lockFile = ../Cargo.lock;

    src = lib.cleanSourceWith {
      src = ../.;
    };

    nativeBuildInputs = with pkgs; [pkg-config];

    buildInputs = with pkgs; [
      lz4
    ];

    doCheck = false;

    buildPhase = ''
      cargo build --release
    '';

    installPhase = ''
      install -Dm755 target/release/mirai $out/bin/mirai
    '';

    meta = with lib; {
      description = "Swww wallpaper manager";
      homepage = "https://github.com/iaverages/mirai";
      license = licenses.mit;
      platforms = platforms.linux;
      mainProgram = "mirai";
    };
  }
