NixOS Configuration Editor
===

[![Built with Nix][builtwithnix badge]][builtwithnix]
[![License: MIT][MIT badge]][MIT]
[![Chat on Matrix][matrix badge]][matrix]

A simple NixOS configuration editor application built with [libadwaita](https://gitlab.gnome.org/GNOME/libadwaita), [GTK4](https://www.gtk.org/), and [Relm4](https://relm4.org/). The goal of this project is to provide a simple graphical tool for modifying and managing desktop NixOS configurations.

## To Do's currently unimplemented

- Handle files in `imports`
- Add an icon

## Things Done

- Set and modify configuration options
    - Validate option types
- Search options
- Indicate which options are set 
- Rebuild system and show errors
- Handle `<name>` and `*` fields in options
- Package polkit policy file
- Add easy widgets for modifying simple options like booleans and strings
    - Plan to add more like lists in the future

## NixOS Installation

```bash
git clone https://github.com/vlinkz/nixos-conf-editor
nix-env -f nixos-conf-editor -i nixos-conf-editor 
```

## Declarative Installation

Head of `configuration.nix`

```nix
{ config, pkgs, lib, ... }:
let
  nixos-conf-editor = (import (pkgs.fetchFromGitHub {
    owner = "vlinkz";
    repo = "nixos-conf-editor";
    rev = "0.0.3";
    sha256 = "0000000000000000000000000000000000000000000000000000";
  })) {};
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
RUST_LOG=trace nixos-conf-editor
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

[builtwithnix badge]: https://img.shields.io/badge/Built%20With-Nix-41439A?style=flat-square&logo=nixos&logoColor=white
[builtwithnix]: https://builtwithnix.org/
[MIT badge]: https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square
[MIT]: https://opensource.org/licenses/MIT
[matrix badge]: https://img.shields.io/badge/matrix-join%20chat-0cbc8c?style=flat-square&logo=matrix&logoColor=white
[matrix]: https://matrix.to/#/#nixos-gui:matrix.org
