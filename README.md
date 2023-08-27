<div align="center">

<img src="data/icons/dev.vlinkz.NixosConfEditor.svg"/>

NixOS Configuration Editor
===

[![Built with Nix][builtwithnix badge]][builtwithnix]
[![License: GPLv3][GPLv3 badge]][GPLv3]
[![Chat on Matrix][matrix badge]][matrix]
[![Chat on Discord][discord badge]][discord]

A simple NixOS configuration editor application built with [libadwaita](https://gitlab.gnome.org/GNOME/libadwaita), [GTK4](https://www.gtk.org/), and [Relm4](https://relm4.org/). The goal of this project is to provide a simple graphical tool for modifying and managing desktop NixOS configurations.

<img src="data/screenshots/multiwindowlight.png#gh-light-mode-only"/>
<img src="data/screenshots/multiwindowdark.png#gh-dark-mode-only"/> 

</div>

## NixOS Installation

Head of `configuration.nix`

if you are on unstable channel or any version after 22.11:
```nix
{ config, pkgs, lib, ... }:
let
  nixos-conf-editor = import (pkgs.fetchFromGitHub {
    owner = "vlinkz";
    repo = "nixos-conf-editor";
    rev = "0.1.1";
    sha256 = "sha256-TeDpfaIRoDg01FIP8JZIS7RsGok/Z24Y3Kf+PuKt6K4=";
  }) {};
in
```
if you are on 22.11:
```nix
{ config, pkgs, lib, ... }:
let
  unstable = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz";
  }) {config = config.nixpkgs.config;};
  nixos-conf-editor = import (pkgs.fetchFromGitHub {
    owner = "vlinkz";
    repo = "nixos-conf-editor";
    rev = "0.1.1";
    sha256 = "sha256-TeDpfaIRoDg01FIP8JZIS7RsGok/Z24Y3Kf+PuKt6K4=";
  }) {pkgs = unstable;};
in
```
Packages:

```nix
environment.systemPackages =
with pkgs; [
  nixos-conf-editor
  # rest of your packages
];
```
For any other method of installation, when rebuilding you will be prompted to authenticate twice in a row

## 'nix profile' installation
```bash
nix profile install github:vlinkz/nixos-conf-editor
```

## 'nix-env' Installation

```bash
git clone https://github.com/vlinkz/nixos-conf-editor
nix-env -f nixos-conf-editor -i nixos-conf-editor 
```

## Single run on an flakes enabled system:
```bash
nix run github:vlinkz/nixos-conf-editor
```

## Single run on non-flakes enabled system:
```bash
nix --extra-experimental-features "nix-command flakes" run github:vlinkz/nixos-conf-editor
```

## Debugging

```bash
RUST_LOG=nixos_conf_editor=trace nixos-conf-editor
```

# Screenshots

<p align="middle">
  <img src="data/screenshots/listviewlight.png#gh-light-mode-only"/>
  <img src="data/screenshots/listviewdark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="data/screenshots/optionlight.png#gh-light-mode-only"/>
  <img src="data/screenshots/optiondark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="data/screenshots/searchlight.png#gh-light-mode-only"/>
  <img src="data/screenshots/searchdark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="data/screenshots/rebuildlight.png#gh-light-mode-only"/>
  <img src="data/screenshots/rebuilddark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="data/screenshots/invalidlight.png#gh-light-mode-only"/>
  <img src="data/screenshots/invaliddark.png#gh-dark-mode-only"/> 
</p>

## Licenses

The icons in [data/icons](data/icons/) contains assets from the [NixOS logo](https://github.com/NixOS/nixos-artwork/tree/master/logo) and are licensed under a [CC-BY license](https://creativecommons.org/licenses/by/4.0/).

[builtwithnix badge]: https://img.shields.io/badge/Built%20With-Nix-41439A?style=for-the-badge&logo=nixos&logoColor=white
[builtwithnix]: https://builtwithnix.org/
[GPLv3 badge]: https://img.shields.io/badge/License-GPLv3-blue.svg?style=for-the-badge
[GPLv3]: https://opensource.org/licenses/GPL-3.0
[matrix badge]: https://img.shields.io/badge/matrix-join%20chat-0cbc8c?style=for-the-badge&logo=matrix&logoColor=white
[matrix]: https://matrix.to/#/#snowflakeos:matrix.org
[discord badge]: https://img.shields.io/discord/1021080090676842506?color=7289da&label=Discord&logo=discord&logoColor=ffffff&style=for-the-badge
[discord]: https://discord.gg/6rWNMmdkgT
