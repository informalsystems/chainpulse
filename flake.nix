{
  description = "ChainPulse";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in {
      packages = forAllSystems (system:
        let
          manifest = (nixpkgs.lib.importTOML ./Cargo.toml).package;
          bin = pkgsFor.${system}.callPackage ./default.nix {};
          # dockerImage = pkgsFor.${system}.callPackage ./docker.nix { inherit bin; };
        in {
          # inherit bin dockerImage;
          default = bin;
        }
      );
      devShells = forAllSystems (system: {
        default = pkgsFor.${system}.callPackage ./shell.nix {};
      });
    };
}
