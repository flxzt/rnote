# Building

First clone the repository and init its submodules
```
git clone https://github.com/flxzt/rnote
cd rnote
git submodule update --init --recursive
```

## Building with Flatpak
There is a flatpak manifest in `build-aux/com.github.flxzt.rnote.Devel.yaml`.

Make sure you have `flatpak` and `flatkpak-builder` installed on your system. You also need the Gnome 43 Runtime, SDK and some extensions:

```bash
flatpak install org.gnome.Platform//43 org.gnome.Sdk//43 org.freedesktop.Sdk.Extension.rust-stable//22.08 org.freedesktop.Sdk.Extension.llvm14//22.08
```

Use Gnome Builder or VSCode with the [flatpak extension](https://marketplace.visualstudio.com/items?itemName=bilelmoussaoui.flatpak-vscode) to build and run the application for you. **This is the easiest and recommended way.**

### Bugs and workarounds
- If you encounter `bwrap: Can't find source path /run/user/1000/doc/by-app/com.github.flxzt.rnote: No such file or directory` when trying to run the flatpak, `xdg-document-portal` did not start yet. Starting it manually with `systemctl start --user xdg-document-portal` should fix it.

- As long as the flatpak is not installed on the system, The DirectoryList in the workspace browser does not update when files are created, removed or changed. It will work in the released flatpak.

- Building the flatpak aborts randomly with status `137` out of memory: Reset the flatpak app-id permissions with `flatpak permission-reset com.github.flxzt.rnote`, so it is able to run in the background. (see [this issue](https://github.com/flatpak/xdg-desktop-portal/issues/478))

### Manual flatpak build
If you don't have an IDE or extension to handle building flatpaks, you can also do it manually:

### Build
Building the app with flatpak is done with:

```bash
flatpak-builder --user flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml
```

Creating a repo:

```bash
flatpak-builder --user --repo=flatpak-repo flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml
```


### Install
Install to the system as user with:

```bash
flatpak-builder --user --install flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml
```

### Run
Then it can be run.
From the build directory:

```bash
flatpak-builder --run flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml rnote
```

Or if it is installed:

```bash
flatpak run com.github.flxzt.rnote
```

## Build with Meson
The flatpak manifest calls the meson build system to build the application.
If a native build on the host is wanted, meson can be called directly.

### Prerequisites

Install all needed dependencies and build tools, e.g. for fedora 37:
```bash
sudo dnf install gcc gcc-c++ clang clang-devel make cmake meson kernel-devel gtk4-devel libadwaita-devel poppler-glib-devel poppler-data alsa-lib-devel
```

Also make sure `rustc` and `cargo` are installed ( see [https://www.rust-lang.org/](https://www.rust-lang.org/) ). Then run:

```bash
meson setup --prefix=/usr _mesonbuild
```
Meson will ask for the user password when needed.

### Configuration
To enable the development profile, set `-Dprofile=devel` as a parameter. Else the `default` profile will be set.

To enable building the `rnote-cli` binary, set `-Dcli=true`.

**Reconfigure**

reconfiguring the meson build options can be done with:

```bash
meson configure -D<option>=<value> _mesonbuild
```

For example if the profile needs to be changed.
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

If meson was configured with a different install prefix path than `/usr`, then GIO needs to be told where the installed gschema is located. this can be done through the `GSETTINGS_SCHEMA_DIR` env variable.

For example to run the application with a custom gschema path: 
`GSETTINGS_SCHEMA_DIR=<prefix_path>/share/glib-2.0/schemas rnote`

### Test
Meson has some tests to validate the desktop, gresources, ... files.

```bash
meson test -v -C _mesonbuild
```

### Uninstall
If you don't like rnote, or decided that is not worth your precious disk space, you can always uninstall it with:

```bash
sudo -E ninja uninstall -C _mesonbuild
```

# Debugging
For a native meson build:
Be sure to configure meson with `-Dprofile=devel` to have a build that includes debugging symbols.
Then configure, compile and install the meson project as outlined above. 

## With VSCode

With the `CodeLLDB` extension can be used to debug, set breakpoints etc. from within the editor.

Create a `tasks.json` file similar to this:

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "meson compile",
            "type": "shell",
            "command": "meson compile -C _mesonbuild"
        },
        {
            "label": "meson install",
            "type": "shell",
            "command": "meson install -C _mesonbuild"
        }
    ]
}
```

and a `launch.json` entry:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "compile and launch debug build of 'rnote'",
            "args": [],
            "program": "${workspaceFolder}/_mesonbuild/target/debug/rnote",
            "preLaunchTask": "meson compile",
            "env": {"RUST_LOG": "rnote=debug"}
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "install and launch debug build of 'rnote'",
            "args": [],
            "program": "${workspaceFolder}/_mesonbuild/target/debug/rnote",
            "preLaunchTask": "meson install",
            "env": {"RUST_LOG": "rnote=debug"}
        }
    ]
}
```

These configurations can then be selected in the `Run and Debug` panel and launched there or through `Run -> Start Debugging`.