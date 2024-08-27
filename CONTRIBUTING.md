# Contributing to Rnote

It is possible to contributes in multiple ways.
They are outlined in this document.

# Maintainers

- @flxzt : Original Author and Core Maintainer - active
- @Doublonmousse : Core Maintainer - active
- @Kneemund : Maintainer - currently inactive

# Bug Reports & Features Requests

Start from the templates located in `.github/ISSUE_TEMPLATE`.

It is recommended to file a bug report or feature request directly through the github web UI
by clicking on `Repository->Issues->New Issue`.
Choose between the different existing templates and **please** fill them out completely as requested.

# Platforms

## Linux

Rnote is mainly developed for Linux and integrates best with the Gnome desktop environment.
However, it should be ensured that it will work well regardless which DE, compositor or distribution is used.

In addition the focus for development and testing is on Wayland,
because X11 has a lot of issues and inconsistencies especially with regards to
pen input which is a integral part of the application.

For more details how to build the application on linux either natively or as flatpak see: [BUILDING.md](./BUILDING.md).

## MacOS

The application is also bundled for MacOS, @dehesselle is active in issues that affect the app bundle.

For more details how to build the application on MacOS
see: [rnote-macos-build.md](./misc/building/rnote-macos-build.md).

## Windows

For windows `mingw64` is used as the development and build environment.
For the installer `innosetup` is used.

It should always be ensured that the app will build in `mingw64`,
but it's not a focus to ensure tight integration with Windows OS.

For more details how to build the application and the installer on Windows
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
Because the app needs to be built through meson, there is a meson target the print lints available:

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

Some unit tests are specified throughout the codebase.

To run them, execute:

```bash
meson compile cargo-test -C _mesonbuild
```

Adding tests can be done like in any other rust crate by adding `tests` modules
and `#[test]` cases directly in code in the same files where the code that should be tested is written.

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

The CI must run successfully in a opened PR to get it merged.
Ideally the optional lint step does not report any warnings.
But because new lints can appear on new clippy versions this is not mandatory.

Please add a short description outlining the changes and the reasons for them.
When adding new features and/or changes in the UI some screenshots or screen captures would be nice.
When it fixes a specific issue, the description should reference the to-be-fixed issue with `fixes #<num>`.

# Build system

For building the application the `cargo` calls are wrapped by meson.
It uses a user specified build directory (e.g. `_mesonbuild`) where the build artifacts will be compiled into.
All needed additional files needed before/after compilation are prepared and placed into it as well.

# Dependencies

Rust dependencies are declared in the root workspace `Cargo.toml`, or if crate-specific
in the individual crates `Cargo.toml` configuration files.

The generated `Cargo.lock` file pins the dependencies to specific versions and is checked in.

All non-rust dependencies are declared in the root `meson.build` file.
You'll see declarations for example for `glib`, `gtk4`, `poppler` and so on.

# Architecture

The codebase is separated into multiple crates that have specific purposes and separate concerns:

- `rnote-compose` : the base crate that is only responsible for supplying basic types needed for a drawing application.
    Things like shapes, paths, pen-path builders, .. . In it is also the code for how to render these primitives
    with `cairo` or rather the `piet` abstraction.
    The dependencies should be kept minimal here. 
- `rnote-engine` : the core crate of the drawing application.
    In it is the entire core logic of the drawing part of the Rnote application.

    It is categorized like this:
    - `rnote-engine/store` : a Entity-Component-System pattern is used there to hold all strokes
    that are produced by the user in a generational Vector and the methods that define the interactions with them.
    - `rnote-engine/document` : information about the entire document (it's dimensions, colors, ..)
    - `rnote-engine/fileformats` : for a stable `.rnote` file format a wrapper for the serialization/deserialization 
        is used to upgrade the files when loading them in.
        Conversions from/to different format's like Xournal++'s `.xopp` are contained in there as well.
    - `rnote-engine/pens` : The user always generates/interacts with strokes through what Rnote internally
        calls `pens`. For example the "Brush" pen produces pen paths, the "Shaper" pen produces geometric shapes,
        the `eraser` pen removes strokes, .. .
    - `rnote-engine`strokes` contain the definition of different types that can be generated or imported
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

the `rnote-compose` and `rnote-compose` crates should be treated as libraries and should contain at least
a bit of documentation for their features and functionality.

The `rnote-cli` and `rnote-ui` crates are "consumer" crates and especially the UI contains a ton of
boilerplate code so in there documentation is not so critical. 

However especially Gtk quirks and workarounds should always be documented in code.
