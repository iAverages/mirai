{
  rust,
  rustPlatform,
  lib,
  pkgs,
  buildWindows ? false,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage {
    pname = "mirai";
    inherit (cargoToml.package) version;

    cargoLock.lockFile = ../Cargo.lock;

    src = let
      src = ../.;
      foldersToIgnore = ["nix" ".jj" ".github" ".git"];
    in
      lib.cleanSourceWith {
        inherit src;
        filter = path: type: let
          relativePath = lib.removePrefix "${toString src}/" path;
        in
          ! (lib.any (ignoredDir: relativePath == ignoredDir || lib.hasPrefix "${ignoredDir}/" relativePath) foldersToIgnore);
      };
    nativeBuildInputs = with pkgs;
      [
        pkg-config
        rust
      ]
      ++ lib.optionals buildWindows [
        pkgsCross.mingwW64.stdenv.cc
      ];

    buildInputs = with pkgs; [
      lz4
    ];

    env = lib.optionalAttrs buildWindows {
      CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
      CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
    };

    doCheck = false;

    buildPhase = ''
      cargo build --release ${lib.optionalString buildWindows "--target=x86_64-pc-windows-gnu"}
    '';

    installPhase = ''
      mkdir -p $out/bin
      install -Dm755 target/release/mirai $out/bin/mirai
    '';

    meta = with lib; {
      description = "Swww wallpaper manager";
      homepage = "https://github.com/iaverages/mirai";
      license = licenses.mit;
      platforms = platforms.linux ++ platforms.windows;
      mainProgram = "mirai";
    };
  }
