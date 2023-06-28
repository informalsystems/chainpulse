{
  description = "ChainPulse";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages = rec {
          bin = pkgs.callPackage ./default.nix {};
          # dockerImage = pkgsFor.${system}.callPackage ./docker.nix { inherit bin; };

          default = bin;
        };

        devShells = {
          default = pkgs.callPackage ./shell.nix {};
        };
      }
    );
}
