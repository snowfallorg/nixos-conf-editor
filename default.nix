{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
}:
pkgs.stdenv.mkDerivation rec {
  pname = "nixos-conf-editor";
  version = "0.1.1";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
    inherit src;
    name = "${pname}-${version}";
    hash = "sha256-47Mw1dvsKw83qJKMA+HWDkzyO7Q4x2IAUOQgTlo8Ylw=";
  };

  nativeBuildInputs = with pkgs; [
    appstream-glib
    desktop-file-utils
    gettext
    git
    meson
    ninja
    pkg-config
    polkit
    wrapGAppsHook4
  ] ++ (with pkgs.rustPlatform; [
    cargoSetupHook
    rust.cargo
    rust.rustc
  ]);

  buildInputs = with pkgs; [
    gdk-pixbuf
    glib
    gnome.adwaita-icon-theme
    gtk4
    gtksourceview5
    libadwaita
    openssl
    vte-gtk4
  ];

  postInstall = ''
    wrapProgram $out/bin/nixos-conf-editor --prefix PATH : '${lib.makeBinPath [ pkgs.pandoc ]}'
  '';
}
