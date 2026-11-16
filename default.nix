{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "instaling-bot";
  version = "1.0.0";

  src = ./.;

  cargoHash = "sha256-gsdIE6FODdMoEAYRA4P6AO4gKBrHLcNWhbY5XA1tDMo=";

  propagatedBuildInputs = with pkgs; [
    firefox
    geckodriver
  ];
})
