{
  lib,
  rustPlatform,
  pkg-config,
  dbus,
}:

let
  cargo = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = cargo.name;
  version = cargo.version;

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ dbus ];

  meta = {
    description = cargo.description;
    homepage = cargo.homepage;
    license = lib.licenses.gpl3Only;
    maintainers = with lib.maintainers; [ samuel-martineau ];
  };
}
