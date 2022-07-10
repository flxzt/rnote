# Build Instructions for macOS

## Dependencies

- `git`
- `cargo` (the Rust toolchain in general)
- `ninja` (backend for `meson`)
- `meson`
- Glib (`glib-2.0`)
- Gio (`gio-2.0`)
- GTK4
- Libadwaita (`libadwaita-1`)
- Poppler (`poppler-glib`)

### Installing Rust

Rust is a necessary dependency and you are recommended to install it via `rustup`. You will also need a C compiler.
```sh
xcode-select --install # install command-line utilities (including the clang compiler)
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh # install rustup
rustup toolchain install stable
```

To learn more about `rustup`, you can check out [the Rust website](https://www.rust-lang.org/tools/install).

### Installing Other Dependencies

[Homebrew](https://brew.sh) is the most widely used package manager for macOS. If you don't have it installed already, you can install it with the following command:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Then, you can install the rest of the required dependencies using the `brew install` command.

```sh
brew install git ninja meson glib appstream-glib gtk4 gstreamer poppler desktop-file-utils
```

## Cloning the Repository

Once you have all the required dependencies, you can clone the reposity by navigating to a directory in which to place the project and then running the following command:

```sh
git clone https://github.com/flxzt/rnote.git
git submodule update --init --recursive
```

Then, navigate into the `rnote` directory and following the build and installation steps.

## Building and Installing the Project

First, we have to setup the build directory. These steps will differ slightly from those listed in the [CONTRIBUTIONS.md](https://github.com/flxzt/rnote/blob/main/CONTRIBUTING.md#build-with-meson) file.

First, we must setup the build directory. The `prefix` will be set to `usr/local` here because `/usr` is [protected by SIP](https://support.apple.com/en-us/HT204899) by default.

```sh
meson setup --prefix=usr/local _mesonbuild
```

Next, we have to build `rnote`.

```sh
meson install -C _mesonbuild
```

Now, we can install the binary and place resource files in their desired locations.

```sh
meson install -C _mesonbuild
```

We must also append the path to the `gschema` file to the `GSETTINGS_SCHEMA_DIR` environment variable. You can simply run the following command before running `rnote` or add it to your `.zshrc`. If you set `prefix` to a different path, you will have to alter the following command accordingly.

```sh
GSETTINGS_SCHEMA_DIR=$GSETTINGS_SCHEMA_DIR:/usr/local/share/glib-2.0/schemas
```

Now, `rnote` should be installed in `/usr/local/bin`.

## Installing as an Application

TODO
