{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            dbus
            pkg-config
            rust-bin.stable.latest.default
          ];
        };
        packages = rec {
          default = bluetui;
          bluetui = let
            cargo = (pkgs.lib.importTOML ./Cargo.toml).package;
          in
            pkgs.rustPlatform.buildRustPackage {
              pname = cargo.name;
              version = cargo.version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              buildInputs = with pkgs; [dbus];
              nativeBuildInputs = with pkgs; [pkg-config];

              meta = {
                description = cargo.description;
                homepage = cargo.homepage;
                license = pkgs.lib.licenses.gpl3Only;
                maintainers = with pkgs.lib.maintainers; [samuel-martineau];
              };
            };
        };
      }
    );
}
