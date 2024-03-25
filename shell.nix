{ pkgs ? import <nixpkgs> {} }:

with pkgs;

pkgs.mkShell rec {
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libxkbcommon
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
