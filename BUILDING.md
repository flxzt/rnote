# Building

First clone the repository and init its submodules
```
git clone https://github.com/flxzt/rnote
cd rnote
git submodule update --init --recursive
```

## Building with Flatpak
There is a flatpak manifest in `build-aux/com.github.flxzt.rnote.Devel.json`.

Make sure you have `flatpak` and `flatkpak-builder` installed on your system.

Use Gnome Builder or VSCode with the [flatpak extension](https://marketplace.visualstudio.com/items?itemName=bilelmoussaoui.flatpak-vscode) to build and run the application for you. **This is the easiest and recommended way.**

### Bugs and workarounds
- If you encounter `bwrap: Can't find source path /run/user/1000/doc/by-app/com.github.flxzt.rnote: No such file or directory` when trying to run the flatpak, `xdg-document-portal` did not start yet. Starting it manually with `systemctl start --user xdg-document-portal` should fix it.

- As long as the flatpak is not installed on the system, The DirectoryList in the workspace browser does not update when files are created, removed or changed. It will work in the released flatpak.

- Building the flatpak aborts randomly with status `137` out of memory: Reset the flatpak app-id permissions with `flatpak permission-reset com.github.flxzt.rnote`, so it is able to run in the background. (see [this issue](https://github.com/flatpak/xdg-desktop-portal/issues/478))

### Prerequisites
If you don't have an IDE or extension to handle building flatpaks, you can also do it manually:

First the Gnome 42 SDK and some extensions are needed:

```bash
flatpak install org.gnome.Platform//43 org.gnome.Sdk//43 org.freedesktop.Sdk.Extension.rust-stable//22.08 org.freedesktop.Sdk.Extension.llvm14
```
### Build
Building the app with flatpak is done with:

```bash
flatpak-builder --user flatpak-app build-aux/com.github.flxzt.rnote.Devel.json
```

Creating a repo:

```bash
flatpak-builder --user --repo=flatpak-repo flatpak-app build-aux/com.github.flxzt.rnote.Devel.json
```


### Install
Install to the system as user with:

```bash
flatpak-builder --user --install flatpak-app build-aux/com.github.flxzt.rnote.Devel.json
```

### Run
Then it can be run.
From the build directory:

```bash
flatpak-builder --run flatpak-app build-aux/com.github.flxzt.rnote.Devel.json rnote
```

Or if it is installed:

```bash
flatpak run com.github.flxzt.rnote
```

## Build with Meson
The flatpak manifest calls the meson build system to build the application.
If a native build on the host is wanted, meson can be called directly.

### Prerequisites

Install all needed dependencies and build tools, e.g. for fedora 36:
```bash
sudo dnf install meson gtk4-devel libadwaita-devel poppler-glib-devel poppler-data alsa-lib-devel
```

Also make sure `rustc` and `cargo` are installed ( see [https://www.rust-lang.org/](https://www.rust-lang.org/) ). Then run:

```bash
meson setup --prefix=/usr _mesonbuild
```
Meson will ask for the user password when needed.

To enable the development profile, set `-Dprofile=devel` as a parameter. Else the `default` profile will be set. ( This can be reconfigured later )

### Compile
Once the project is configured, it can be compiled with:

```bash
meson compile -C _mesonbuild
```

The compiled binary should now be here: `./_mesonbuild/target/release/rnote`.

### Install
Installing the binary into the system can be done with:

```bash
meson install -C _mesonbuild
```

This places the files in the specified prefix and their subpaths. The binary should now be in `/usr/bin` (and therefore in PATH)

### Test
Meson has some tests to validate the desktop, gresources, ... files.

```bash
meson test -v -C _mesonbuild
```

### Reconfigure
reconfiguring the meson build can be done with:

```bash
meson configure -Dprofile=default _mesonbuild
```

For example if the profile needs to be changed.

### Uninstall
If you don't like rnote, or decided that is not worth your precious disk space, you can always uninstall it with:

```bash
sudo -E ninja uninstall -C _mesonbuild
```

# Debugging
For a native meson build:

Change these lines in `build-aux/cargo.sh`:
```bash
    echo -e "\n--- DEVEL PROFILE ---\n"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml && \
        cp "$CARGO_TARGET_DIR"/debug/"$APP_BIN" "$OUTPUT"
```

Then configure, compile and install the meson project as outlined above. Be sure to configure meson with -Dprofile=devel.

## With VSCode
Create a `launch.json` entry similar to this:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "launch debug build of 'rnote'",
            "args": [],
            "program": "${workspaceFolder}/target/debug/rnote"
        },
    ]
}
```

In vscode the `CodeLLDB` extension can be used to debug from within the editor.
