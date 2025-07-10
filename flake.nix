{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";

  outputs = {
    self,
    nixpkgs,
  }: {
    packages =
      nixpkgs.lib.genAttrs
      [
        "x86_64-linux"
        "aarch64-linux"
      ]
      (system: rec {
        bluetui = nixpkgs.legacyPackages.${system}.callPackage ./package.nix {};
        default = bluetui;
      });

    devShells.x86_64-linux.default = let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in
      pkgs.mkShell {
        buildInputs = [
          self.packages.x86_64-linux.bluetui
        ];
      };
  };
}
