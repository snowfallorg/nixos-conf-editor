{
  pkgs ? import <nixpkgs> {},
  lib ? import <nixpkgs/lib>,
}:
pkgs.stdenv.mkDerivation rec {
  pname = "nixos-conf-editor";
  version = "0.0.3";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
    inherit src;
    name = "${pname}-${version}";
    hash = "sha256-cdZRirlmQrMuQvXc/K/WlRqcgyoAte6dgRE7mmLrTjw=";
  };

  nativeBuildInputs = with pkgs; [
    appstream-glib
    polkit
    gettext
    desktop-file-utils
    meson
    ninja
    pkg-config
    git
    wrapGAppsHook4
  ] ++ (with pkgs.rustPlatform; [
    cargoSetupHook
    rust.cargo
    rust.rustc
  ]);

  buildInputs = with pkgs; [
    gdk-pixbuf
    glib
    gtk4
    gtksourceview5
    libadwaita
    openssl
    wayland
    gnome.adwaita-icon-theme
  ];

  postInstall = ''
    wrapProgram $out/bin/nixos-conf-editor --prefix PATH : '${lib.makeBinPath [ pkgs.pandoc ]}'
  '';
}
