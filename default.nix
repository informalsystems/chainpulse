{ pkgs ? import <nixpkgs> {} }:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = manifest.name;
    version = manifest.version;

    meta = with pkgs.lib; {
      description = manifest.description;
      homepage = manifest.homepage;
      license = manifest.license;
    };

    cargoLock.lockFile = ./Cargo.lock;

    src = pkgs.lib.cleanSource ./.;

    buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ];
  }
