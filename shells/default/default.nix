{ mkShell
, cairo
, cargo
, clippy
, desktop-file-utils
, gdk-pixbuf
, gettext
, gobject-introspection
, graphene
, gtk4
, gtksourceview5
, libadwaita
, meson
, ninja
, openssl
, pandoc
, pango
, pkg-config
, polkit
, rust
, rust-analyzer
, rustc
, rustfmt
, vte-gtk4
, wrapGAppsHook4
}:

mkShell {
  buildInputs = [
    cairo
    cargo
    clippy
    desktop-file-utils
    gdk-pixbuf
    gettext
    gobject-introspection
    graphene
    gtk4
    gtksourceview5
    libadwaita
    meson
    ninja
    openssl
    pandoc
    pango
    pkg-config
    polkit
    rust-analyzer
    rustc
    rustfmt
    vte-gtk4
    wrapGAppsHook4
  ];
  RUST_SRC_PATH = "${rust.packages.stable.rustPlatform.rustLibSrc}";
}
