{ pkgs ? import <nixpkgs> { }
, lib ? import <nixpkgs/lib>
, libadwaita-git ? null
}:
let
  libadwaita =
    if libadwaita-git != null
    then libadwaita-git
    else
      pkgs.libadwaita.overrideAttrs (oldAttrs: rec {
        version = "1.2.0";
        src = pkgs.fetchFromGitLab {
          domain = "gitlab.gnome.org";
          owner = "GNOME";
          repo = "libadwaita";
          rev = version;
          hash = "sha256-3lH7Vi9M8k+GSrCpvruRpLrIpMoOakKbcJlaAc/FK+U=";
        };
      });
in
pkgs.stdenv.mkDerivation rec {
  pname = "nixos-conf-editor";
  version = "0.0.5";

  src = [ ./. ];

  cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
    inherit src;
    name = "${pname}-${version}";
    hash = "sha256-HbNhtgYqx2RquLA81NAGJNzzaC3GBSvjnwpb9NmqPjU=";
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
