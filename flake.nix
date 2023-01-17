{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        name = "nixos-conf-editor";
      in
      rec
      {
        packages.${name} = pkgs.callPackage ./default.nix {
          inherit (inputs);
        };

        # `nix build`
        defaultPackage = packages.${name};

        # `nix run`
        apps.${name} = utils.lib.mkApp {
          inherit name;
          drv = packages.${name};
        };
        defaultApp = apps.${name};

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
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
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
}
