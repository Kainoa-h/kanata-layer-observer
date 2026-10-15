{ lib, rustPlatform }:

let
  manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./LICENSE
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "Observe Kanata layer changes and invoke a callback";
    homepage = "https://github.com/Kainoa-h/kanata-layer-observer";
    license = lib.licenses.lgpl3Only;
    mainProgram = "kanata-layer-observer";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
}
