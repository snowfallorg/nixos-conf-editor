{ stdenv
, lib
, appstream-glib
, cargo
, desktop-file-utils
, gdk-pixbuf
, gettext
, git
, glib
, gnome
, gtk4
, gtksourceview5
, libadwaita
, meson
, ninja
, openssl
, pandoc
, pkg-config
, polkit
, rustc
, rustPlatform
, vte-gtk4
, wrapGAppsHook4
}:
stdenv.mkDerivation rec {
  pname = "nixos-conf-editor";
  version = "0.1.2";

  src = [ ../.. ];

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../../Cargo.lock;
  };

  nativeBuildInputs = [
    appstream-glib
    desktop-file-utils
    gettext
    git
    meson
    ninja
    pkg-config
    polkit
    wrapGAppsHook4
  ] ++ (with rustPlatform; [
    cargo
    cargoSetupHook
    rustc
  ]);

  buildInputs = [
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
    wrapProgram $out/bin/nixos-conf-editor --prefix PATH : '${lib.makeBinPath [ pandoc ]}'
  '';
}
