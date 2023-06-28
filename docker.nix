{ system ? "x86_64-linux"
, pkgs ? import <nixpkgs> { inherit system; }
, bin ? pkgs.callPackage ./default.nix { inherit pkgs; }
}:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.dockerTools.buildImage {
    name = manifest.name;
    tag = manifest.version;
    copyToRoot = [ bin ];
    config = {
      Cmd = [ "${bin}/bin/chainpulse" ];
    };
  }
