{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";

  outputs =
    { self, nixpkgs }:
    {
      packages =
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
          ]
          (system: rec {
            bluetui = nixpkgs.legacyPackages.${system}.callPackage ./package.nix { };
            default = bluetui;
          });
    };
}
