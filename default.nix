{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "instaling-bot";
  version = "1.0.0";
  src = ./.;

  cargoHash = "sha256-EXjkFwQ//CT5qfdt2lFQV2o69wfHBd9ZAtaUszjlkiA=";

  nativeBuildInputs = with pkgs; [
    pkg-config
    makeWrapper
  ];
  buildInputs = with pkgs; [
    aws-lc
    zstd
  ];

  installPhase = ''
    mkdir -p $out/bin
    cp instaling-bot $out/bin/instaling-bot
    wrapProgram $out/bin/instaling-bot \
      --prefix PATH : ${pkgs.lib.makeBinPath (with pkgs; [ firefox geckodriver ])}
  '';
})
