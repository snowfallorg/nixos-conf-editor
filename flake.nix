{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem
      (system:
       let 
          name = "nixos-conf-editor";
          pkgs = import nixpkgs { inherit system; };
          naersk-lib = naersk.lib."${system}";
        in rec {
          packages.${name} = naersk-lib.buildPackage {
            pname = "${name}";
            root = ./.;
            copyLibs = true;
            buildInputs = with pkgs; [
              cairo
              gdk-pixbuf
              gobject-introspection
              graphene
              gtk4
              gtksourceview5
              libadwaita
              openssl
              pandoc
              pango
              pkgconfig
              wrapGAppsHook
            ];
            postInstall = ''
               wrapProgram $out/bin/nixos-conf-editor --prefix PATH : '${nixpkgs.lib.makeBinPath [ pkgs.pandoc ]}'
            '';

          };

          # `nix build`
          defaultPackage = packages.${name};

          # `nix run`
          apps.${name} = utils.lib.mkApp {
            inherit name;
            drv = packages.${name};
          };
          defaultApp = packages.${name};

          # `nix develop`
          devShells = {
            default = pkgs.mkShell {
              nativeBuildInputs = 
                with pkgs; [
                  rustc
                  cargo
                  cairo
                  gdk-pixbuf
                  gobject-introspection
                  graphene
                  gtk4
                  gtksourceview5
                  libadwaita
                  openssl
                  pandoc
                  pango
                  pkgconfig
                  wrapGAppsHook
               ] ;
            };
          };
        }
      );
}
