NixOS Configuration Editor
===

[![Built with Nix][builtwithnix badge]][builtwithnix]
[![License: MIT][MIT badge]][MIT]

A simple NixOS configuration editor application built with [libadwaita](https://gitlab.gnome.org/GNOME/libadwaita), [GTK4](https://www.gtk.org/), and [Relm4](https://relm4.org/). The goal of this project is to provide a simple graphical tool for modifying and managing desktop NixOS configurations.

## To Do's currently unimplemented

- Handle `<name>` and `*` fields in options
- Handle files in `imports`
- Add easy widgets for modifying simple options like booleans and strings
- Add an icon
- Package polkit policy file

## Things Done

- Set and modify configuration options
    - Validate option types
- Search options
- Indicate which options are set 
- Rebuild system and show errors

## Usage
This tool is extremely new, so there will likely be lots of errors. If you run into an error, please report the issue!

On an flakes enabled system:
```bash
nix run github:vlinkz/nixos-conf-editor
```

On non-flakes enabled system:
```bash
nix --extra-experimental-features "nix-command flakes" run github:vlinkz/nixos-conf-editor
```

# Screenshots

<p align="middle">
  <img src="screenshots/listviewlight.png#gh-light-mode-only"/>
  <img src="screenshots/listviewdark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="screenshots/optionlight.png#gh-light-mode-only"/>
  <img src="screenshots/optiondark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="screenshots/searchlight.png#gh-light-mode-only"/>
  <img src="screenshots/searchdark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="screenshots/rebuildlight.png#gh-light-mode-only"/>
  <img src="screenshots/rebuilddark.png#gh-dark-mode-only"/> 
</p>

<p align="middle">
  <img src="screenshots/invalidlight.png#gh-light-mode-only"/>
  <img src="screenshots/invaliddark.png#gh-dark-mode-only"/> 
</p>

[builtwithnix badge]: https://img.shields.io/badge/Built%20With-Nix-41439A?style=flat-square&logo=nixos&logoColor=white
[builtwithnix]: https://builtwithnix.org/
[MIT badge]: https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square
[MIT]: https://opensource.org/licenses/MIT