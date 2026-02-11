{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        bluetui = pkgs.callPackage ./package.nix {inherit rustPlatform;};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            dbus
            pkg-config
            rustToolchain
            self.packages.${system}.default
          ];
        };
        packages = {
          default = bluetui;
          inherit bluetui;
        };
        legacyPackages = pkgs.extend (final: prev: {
          bluetui = final.callPackage ./package.nix {
            rustPlatform = final.makeRustPlatform {
              cargo = final.rust-bin.stable.latest.default;
              rustc = final.rust-bin.stable.latest.default;
            };
          };
        });
      }
    );
}