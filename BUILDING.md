# Prerequisites
First install git, clone the repository and init its submodules
```bash
sudo dnf install git
git clone https://github.com/flxzt/rnote
cd rnote
git submodule update --init --recursive
```

# Building with Flatpak vs Meson
This project can be compiled in two different ways depending on your needs: flatpak or meson.

Flatpak is a sandboxed environment/distribution used for building and running applications in a way that is more user
friendly and cross platform. When using flatpak to build an application, flatpak creates a sandboxed environment
tailored to exactly what the application needs. This makes it much easier to compile and run an application without
issues.

Meson is the build system that Rnote uses for building the application. It is called when the flatpak is built. It is
also possible to use meson directly on the host. Because it is building on the host machine, it may require more upfront
work managing the host environment, but then compiling changes to the codebase can be much faster since it does not
require rebuilding a sandboxed environment.

# Building with Flatpak
There is a flatpak manifest in `build-aux/com.github.flxzt.rnote.Devel.yaml`.

Make sure you have `flatpak` and `flatpak-builder` installed on your system.


For Fedora:
```bash
sudo dnf install flatpak flatpak-builder
```

Flathub needs to be added as remote repository:

```bash
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

The flatpak Gnome Runtime, SDK and some extensions are needed:

```bash
flatpak install org.gnome.Platform//46 org.gnome.Sdk//46 org.freedesktop.Sdk.Extension.rust-stable//23.08 \
org.freedesktop.Sdk.Extension.llvm17//23.08
```

Use Gnome Builder or VSCode with the
[flatpak extension](https://marketplace.visualstudio.com/items?itemName=bilelmoussaoui.flatpak-vscode) to build and run
the application for you. **This is the easiest and recommended way.**

## Bugs and workarounds
- If you encounter
    `bwrap: Can't find source path /run/user/1000/doc/by-app/com.github.flxzt.rnote: No such file or directory` when
    trying to run the flatpak, `xdg-document-portal` did not start yet. Starting it manually with
    `systemctl start --user xdg-document-portal` should fix it.
- As long as the flatpak is not installed on the system, The DirectoryList in the workspace browser does not update when
    files are created, removed or changed. It will work in the released flatpak.
- Building the flatpak aborts randomly with `status 137 out of memory`: Reset the flatpak App-ID permissions by
    executing `flatpak permission-reset com.github.flxzt.rnote`, so the build is able to run in the background.
    (see [this issue](https://github.com/flatpak/xdg-desktop-portal/issues/478))

## Manual flatpak build
If you don't have an IDE or extension to handle building flatpaks, you can also do it manually:

### Build
Building the app by executing:

```bash
flatpak-builder --user flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml
```

Create a repo:

```bash
flatpak-builder --user --repo=flatpak-repo flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml
```

### Install
Install to the system as user with:

```bash
flatpak-builder --user --install flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml
```

### Run
Then it can be run. From the build directory:

```bash
flatpak-builder --run flatpak-app build-aux/com.github.flxzt.rnote.Devel.yaml rnote
```

Or if it is installed:

```bash
flatpak run com.github.flxzt.rnote
```

# Build with Meson
The flatpak manifest calls the meson build system to build the application.
If a native build on the host is wanted, meson can be called directly.

## Prerequisites
Install all needed dependencies and build tools, e.g. for Fedora:

```bash
sudo dnf install gcc gcc-c++ clang clang-devel python3 make cmake meson git appstream-util gettext desktop-file-utils \
shared-mime-info kernel-devel gtk4-devel libadwaita-devel poppler-glib-devel poppler-data alsa-lib-devel
```

For debian based distros:

```bash
sudo apt install build-essential clang libclang-dev python3 make cmake meson git appstream-util gettext \
desktop-file-utils shared-mime-info libgtk-4-dev libadwaita-1-dev libpoppler-glib-dev libasound2-dev
```

Also make sure `rustc` and `cargo` are installed ( see [https://www.rust-lang.org/](https://www.rust-lang.org/) ).
Then run:

```bash
meson setup --prefix=/usr _mesonbuild
```
Meson will ask for the user password when needed.

## Configure
To enable the development profile, set `-Dprofile=devel` as a parameter in the setup.
Else the `default` profile will be set.

To enable building the `rnote-cli` binary, set `-Dcli=true`.

## Reconfigure
Reconfiguring the meson build options can be done with:

```bash
meson configure -D<option>=<value> _mesonbuild
```

For example if the profile needs to be changed.
## Compile
Once the project is configured, it can be compiled with:

```bash
meson compile -C _mesonbuild
```

The compiled binary should now be here: `./_mesonbuild/target/release/rnote`.

Note that if an older version of rnote has previously been installed, the old `gschema` file, which defines the
applications settings, will still be used. This can cause problems, when the schema used by the development version are
different from the ones installed locally:
```
Settings schema 'com.github.flxzt.rnote' does not contain a key named '...'
```
In this case you can install the new version of rnote to update the `gschema`.

## Install
Installing the binary into the system can be done with:

```bash
meson install -C _mesonbuild
```

This places the files in the specified prefix and their subpaths. The binary should now be in `/usr/bin`
(and therefore in PATH)
If meson was configured with a different install prefix path than `/usr`, then GIO needs to be told where the installed
gschema is located. this can be done through the `GSETTINGS_SCHEMA_DIR` env variable.

For example to run the application with a custom gschema path: 
`GSETTINGS_SCHEMA_DIR=<prefix_path>/share/glib-2.0/schemas rnote`

## Test
Meson has some tests to validate the desktop, gresources, ... files.

```bash
meson test -v -C _mesonbuild
```

## Uninstall
If you don't like rnote, or decided that is not worth your precious disk space, you can always uninstall it with:

```bash
sudo -E ninja uninstall -C _mesonbuild
```

## Custom Targets
There are various custom targets available. Use them like this:

```bash
meson compile <custom target> -C _mesonbuild
```

| target | explanation |
|---|---|
| rnote-pot | Regenerate the po template file. Provided by `i18n` module. |
| rnote-update-po | Update the po files from the template. Provided by the `i18n` module. |
| rnote-gmo | Compile the po files. Provided by the `i18n` module. |
| cargo-fmt-check | Check the code formatting |
| cargo-test | Run all unit and integration tests |
| cargo-clean | Clean artifacts that cargo has generated |
| ui-cargo-check | Run cargo check for the ui package |
| ui-cargo-clippy | Run clippy for the ui package |
| ui-cargo-doc | Generate docs for the ui package (also checks documentation formatting) |
| ui-cargo-build | Build the ui |
| cli-cargo-check | Run cargo check for the cli package |
| cli-cargo-clippy | Run clippy for the cli package |
| cli-cargo-doc | Generate docs for the cli package (also checks documentation formatting) |
| cli-cargo-build | Build the cli |
| build-installer | Build the installer (only functional on windows-msys2 and when the ui option is enabled) |

# Debugging
For a native meson build:
Be sure to configure meson with option `-Dprofile=devel` to have a build that includes debugging symbols.
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

These configurations can then be selected in the `Run and Debug` panel and launched there or through
`Run -> Start Debugging`.
