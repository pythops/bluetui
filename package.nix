{
  lib,
  rustPlatform,
  dbus,
  pkg-config,
}:
let
  cargo = lib.importTOML ./Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = cargo.package.name;
  version = cargo.package.version;
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  buildInputs = [dbus];
  nativeBuildInputs = [pkg-config];

  meta = {
    description = cargo.package.description;
    homepage = cargo.package.homepage;
    license = lib.licenses.gpl3Only;
    maintainers = with lib.maintainers; [samuel-martineau];
  };
}
