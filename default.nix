{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
}:
pkgs.stdenv.mkDerivation rec {
  pname = "nixos-conf-editor";
  version = "0.1.1";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "nix-data-0.0.2" = "sha256-8j3j+Bk90eBBqQkguwU0tTtMU7Zecjhr1SbW5hnptFY=";
    };
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
