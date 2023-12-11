# Build Instructions for Windows

## Prerequisites

-   Install [MSYS2](https://www.msys2.org/).
-   Install [Rust](https://www.rust-lang.org/).
-   OPTIONAL: Install [Inno Setup](https://jrsoftware.org/isinfo.php) for building the installer.

> The following instructions assume that the default installation directories were used.

The MSYS2 binary directories, namely `C:\msys64\mingw64\bin` and `C:\msys64\usr\bin`, must be added to the system
environment variable `Path`.

### Dependencies

In order to install the necessary dependencies, run the following command in a MSYS2 terminal.

```bash
pacman -S git mingw-w64-x86_64-xz mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gcc mingw-w64-x86_64-clang \
mingw-w64-x86_64-toolchain mingw-w64-x86_64-autotools mingw-w64-x86_64-make mingw-w64-x86_64-cmake \
mingw-w64-x86_64-meson mingw-w64-x86_64-diffutils mingw-w64-x86_64-desktop-file-utils mingw-w64-x86_64-appstream-glib \
mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-poppler mingw-w64-x86_64-poppler-data \
mingw-w64-x86_64-angleproject
```

### Configuration

Add the Rust binary directory to the MSYS2 `PATH` by adding the following line to `~/.bashrc`.

```bash
export PATH=$PATH:/c/Users/$USER/.cargo/bin
```

If you installed Inno Setup, append `:/c/Program\ Files\ \(x86\)/Inno\ Setup\ 6` to the line above.

Next, Rust's toolchain needs to be changed.

```bash
rustup toolchain install stable-gnu
rustup default stable-gnu
```

To be able to create symlinks present in the project when it's sources are cloned, make sure that the `Developer Mode`
in Windows is enabled. It doesn't say it, but it enables permissions for users to create symlinks.

Finally, clone the repository somewhere and initialize the submodules.

```bash
git clone https://github.com/flxzt/rnote
git submodule update --init --recursive
```

For unknown reasons, `libpthread.a` **and** `libpthread.dll.a` exist in `/mingw64/lib/` and rustc apparently wants to
link with both, resulting in "multiple definitions of pthread\_..." linker errors.
To solve this (in a very hacky way), rename `libpthread.dll.a` to `libpthread.dll.a.bak`.

```bash
mv /mingw64/lib/libpthread.dll.a /mingw64/lib/libpthread.dll.a.bak
```

## Building the Application

In the directory that you cloned Rnote into, run the following command to setup meson.

```bash
meson setup --prefix=C:/msys64/mingw64 _mesonbuild
```

Then, the project can be compiled...

```bash
meson compile -C _mesonbuild
```

...and installed.

```bash
meson install -C _mesonbuild
```

The installed binary can now be executed. It is located at `C:\msys64\mingw64\bin\rnote.exe` and depends on the
environment provided by MSYS2, so it is not portable.

## Building the Installer

In order to build the installer, run the commands below.

```bash
meson compile rnote-gmo -C _mesonbuild
meson compile build-installer -C _mesonbuild
```

If successful, the generated installer will be located at `_mesonbuild/rnote-win-installer.exe`.

If you did not install MSYS2 into the default directory (`C:\msys64`), then you will have to adjust the meson option
called `msys-path` prior to building.

```bash
meson configure -Dmsys-path='C:\path\to\msys64' _mesonbuild
```

Likewise, you can adjust the output name of the installer using the `win-installer-name` option
(which defaults to `rnote_installer`).
