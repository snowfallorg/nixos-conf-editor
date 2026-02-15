{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    snowfall-lib = {
      url = "github:snowfallorg/lib";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat.url = "github:edolstra/flake-compat";
  };

  outputs = inputs:
    inputs.snowfall-lib.mkFlake {
      inherit inputs;
      alias.packages.default = "nixos-conf-editor";
      src = ./.;
    };
}
