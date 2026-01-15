# Contributing to Rnote

This document outlines the various ways of contributing to Rnote
and gives an overview of its structure and the tooling used.

# Maintainers

- @flxzt : Original Author and Maintainer
- @Doublonmousse : Maintainer
- @Kneemund : Maintainer

# Bug Reports & Features Requests

Start from the templates located in `.github/ISSUE_TEMPLATE`.

It is recommended to file a bug report or feature request directly through the github web UI
by clicking on `Repository->Issues->New Issue`.
Choose between the different existing templates and **please** fill them out completely as requested.

# Platforms

## Linux

Rnote is mainly developed for Linux and integrates best with the Gnome desktop environment.
The application should nonetheless be ensured to function properly
regardless of which DE, compositor, or distribution is used.

In addition the focus for development and testing is on Wayland,
at this point X11 has a lot of issues and inconsistencies especially with regards to
pen input which is an integral part of the application.
This is why X11 is now considered unsupported.

For more details on how to build the application on linux either natively or as flatpak see: [BUILDING.md](./BUILDING.md).

## MacOS

The application is also bundled for MacOS, @dehesselle is active in issues that affect the app bundle.

For more details on how to build the application on MacOS
see: [rnote-macos-build.md](./misc/building/rnote-macos-build.md).

## Windows

For windows `mingw64` is used as the development and build environment.
For the installer `innosetup` is used.

It should always be ensured that the app will build in `mingw64`,
however tight integration with the Windows OS is not a priority.

For more details on how to build the application and the installer on Windows
see: [rnote-windows-build.md](./misc/building/rnote-windows-build.md).

# Translations

A great way to contribute to the project without writing code is adding a new
or start maintaining an existing translation language.
The translations files are located in `crates/rnote-ui/po/`.

 Creating translations for new languages or updating existing ones can be done in multiple ways:
- take the `rnote.pot` file and generate a new `.po` translation file from it, for example with "Poedit".
    Add the new translation language to `LINGUAS` and submit a PR with the changed files.
- use [weblate](https://hosted.weblate.org/projects/rnote/repo/) for an easy way to translate in the browser
    without having to deal with git.

# Code Style

## Formatting

For formatting `rustfmt` is used. It picks up the formatting configuration file `rustfmt.toml`.

To check the formatting run:

```bash
cargo fmt --check
```

And to directly apply

```bash
cargo fmt
```
The formatting is also checked in the CI and a prerequisite for merging additional or changed code. 

## Lints

For linting `clippy` is used.
Because the app needs to be built through meson, there is a meson target that prints available lints:

For the UI

```bash
meson compile ui-cargo-clippy -C _mesonbuild
```

For the CLI

```bash
meson compile cli-cargo-clippy -C _mesonbuild
```

If consciously considered clippy warnings can also disabled in code by using `#[allow(clippy::<lint-name>)].
However this must be justified when getting new code in.

## Pre-Commit hooks

Per default on an initial build git pre-commit hooks are installed to ensure the outlined style consistency.
Check out [pre-commit.hook](hooks/pre-commit.hook) to see what the hook will do in detail.

# Tests

## Unit Tests

Some unit tests are added throughout the codebase.
First, install [cargo-nextest](https://nexte.st/).
To run the tests execute:

```bash
meson compile cargo-test -C _mesonbuild
```

Just like in any other rust crate, tests can be added by declaring a tests module prefixed with the #[cfg(test)]
attribute, then adding test functions prefixed by the #[test] attribute.
Tests should be as closely coupled to the code they target as reasonably possible
and in most cases should reside in the same source file.

## Data / Package File Validation

To check the style and correctness of other data/auxiliary files like the `.desktop`
or `metainfo.xml` AppData definition file execute:

```bash
meson test -C _mesonbuild
```

# Contributing Code

All code additions should go through a PR->Review cycle.
The core maintainers can also push directly to main but should only do that in case of
trivial changes and fixes.

The CI must run successfully in an opened PR to get it merged.
Ideally the optional lint step does not report any warnings.
But because new lints can appear on new clippy versions this is not mandatory.

Please add a short description outlining the changes and the reasons for them.
When adding new features and/or changes in the UI some screenshots or screen captures would be nice.
When it fixes a specific issue, the description should reference the to-be-fixed issue with `fixes #<num>`.

# Build system

For building the application the `cargo` calls are wrapped by meson.
It uses a user specified build directory (e.g. `_mesonbuild`) where the build artifacts will be compiled into.
All additional files needed before/after compilation are prepared and placed into it as well.

# Dependencies

Rust dependencies are declared in the root workspace `Cargo.toml`, or if crate-specific
in the individual crate's `Cargo.toml` configuration files.

The generated `Cargo.lock` file pins the dependencies to specific versions and is checked in.

All non-rust dependencies are declared in the root `meson.build` file.
For example, you'll find declarations for dependencies like `glib` and `gtk4`.

# Architecture

The codebase is separated into multiple crates that have specific purposes and separate concerns:

- `rnote-compose` : the base crate that is only responsible for supplying basic types needed for a drawing application.
    Things like shapes, paths, pen-path builders, etc. In this crate is also the implementation for how to render
    these primitives with `cairo` or rather the `piet` abstraction.
    The dependencies should be kept minimal here. 
- `rnote-engine` : the core crate of the drawing application.
    In it is the entire core logic of the drawing part of the Rnote application.

    It is categorized like this:
    - `rnote-engine/store` : an Entity-Component-System pattern is used there to hold all strokes
    that are produced by the user in a generational Vector and the methods that define the interactions with them.
    - `rnote-engine/document` : information about the entire document (it's dimensions, colors, ..)
    - `rnote-engine/fileformats` : dictates the current stable Rnote file format,
        and implements the methods required to load and save itself;
        additionally contains the code required to convert from and into other formats
        (notably Xournal++'s `.xopp` format and older versions of the Rnote file format).
    - `rnote-engine/pens` : The user always generates/interacts with strokes through what Rnote internally
        calls `pens`. For example the "Brush" pen produces pen paths, the "Shaper" pen produces geometric shapes,
        the `eraser` pen removes strokes, .. .
    - `rnote-engine/strokes` contain the definition of different types that can be generated or imported
        into the engine. There are "brush strokes", "shape strokes" but also vector and rasterized images
        are represented as a "stroke type".
    
    The main "Engine" type is responsible for keeping an undo-stack and utilizes the Clone-On-Write datastructure
    of the "store" to achieve that.

    There are also smaller utilities and features like the "Camera" which is responsible for the canvas viewport,
    "AudioPlayer" to play pen sounds when enabled, .. .

- `rnote-cli` : basic CLI frontend that takes the engine as dependency and uses the "clap" crate.
    Intended to be used by power-users for automating format conversions or exports and other miscellaneous tasks.
    But it also plays a role in verifying the stability of the file format - it's test subcommand 
    is used in the CI to check whether `.rnote` files in different versions can still be imported successfully.

- `rnote-ui` : the UI frontend built with Gtk4 and Libadwaita.
    Most of the code here is glib `Object`'s or Gtk `Widget`s.
    The application is represented by `RnApp`, the main application window by `RnAppWindow`
    and the Canvas by `RnCanvas`. The canvas has one instance per tab and holds the engine.

# Documentation

the `rnote-compose` and `rnote-engine` crates should be treated as libraries and should contain at least
a bit of documentation for their features and functionality.

The `rnote-cli` and `rnote-ui` crates are "consumer" crates and especially the UI contains a ton of
boilerplate code so in there documentation is not so critical. 

However especially Gtk quirks and workarounds should always be documented in code.
