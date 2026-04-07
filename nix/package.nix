{
  rust,
  rustPlatform,
  lib,
  pkgs,
  buildWindows ? false,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  cargoTarget =
    if buildWindows
    then "x86_64-pc-windows-gnu"
    else null;
  targetDir =
    if buildWindows
    then "target/${cargoTarget}/release"
    else "target/release";
  binaryName =
    if buildWindows
    then "mirai.exe"
    else "mirai";
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
      CC = "${pkgs.stdenv.cc}/bin/cc";
      HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
      CC_x86_64_unknown_linux_gnu = "${pkgs.stdenv.cc}/bin/cc";
      CC_x86_64_pc_windows_gnu = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
      RUSTFLAGS = "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
    };

    doCheck = false;

    buildPhase = ''
      cargo build --release ${lib.optionalString buildWindows "--target=${cargoTarget}"}
    '';

    installPhase = ''
      mkdir -p $out/bin
      install -Dm755 ${targetDir}/${binaryName} $out/bin/${binaryName}
    '';

    meta = with lib; {
      description = "Swww wallpaper manager";
      homepage = "https://github.com/iaverages/mirai";
      license = licenses.mit;
      platforms = platforms.linux ++ platforms.windows;
      mainProgram = "mirai";
    };
  }
