{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "instaling-bot";
  version = "1.0.0";

  src = ./.;

  cargoHash = "sha256-9atn5qyBDy4P6iUoHFhg+TV6Ur71fiah4oTJbBMeEy4=";

  propagatedBuildInputs = with pkgs; [
    firefox
    geckodriver
  ];

  # meta = {
  #   description = "";
  #   license = lib.licenses.unlicense;
  #   maintainers = [ ];
  # };
})
