{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
	propagatedBuildInputs = with pkgs; [
		geckodriver
		firefox
	];
}
