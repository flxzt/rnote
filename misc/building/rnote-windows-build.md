# Build Instructions for Windows

## Prerequisites

- Install [MSYS2](https://www.msys2.org/).
- Install [Rust](https://www.rust-lang.org/).
- Optional for building the installer: Install [Inno Setup](https://jrsoftware.org/isinfo.php).

> The following instructions assume that the default installation directories were used.

### Path configuration 

Add the Rust binary directory to the MSYS2 `PATH` by adding the following line to `~/.bashrc`.

```bash
export PATH="$PATH:/c/Users/<user>/.cargo/bin"
```

If you installed Inno Setup, append `:/c/Program Files (x86)/Inno Setup 6` to the path as well.

### Dependencies

To install dependencies, install `just` and run the `prerequisites-win` command inside the MSYS2 terminal
```bash
cargo install --locked just
just prerequisites-win
```
Or run the corresponding `prerequisites-win` section in [the justfile](../../justfile) in a MSYS2 terminal manually.

### Configuration

Next, Rust's toolchain needs to be changed.

```bash
rustup toolchain install stable-gnu
rustup default stable-gnu
```

To be able to create symlinks present in the project when its sources are cloned, make sure that the `Developer Mode`
in Windows is enabled.
It doesn't say it, but it enables permissions for users to create symlinks.

Then clone the repository and initialize the submodules.

```bash
git clone -c core.symlinks=true https://github.com/flxzt/rnote
cd rnote/
git submodule update --init --recursive
```

Or (from the mingw64 terminal):

```bash
MSYS=winsymlinks:native git clone https://github.com/flxzt/rnote.git
git submodule update --init --recursive
```

Verify that you see in `/crates/rnote-ui/po` the four files zh_CN.po, zh_HK.po, zh_SG.po and zh_TW.po as symlinks
(and not as a text file with a single line inside).

For unknown reasons, `libpthread.a` **and** `libpthread.dll.a` exist in `/mingw64/lib/`´
and rustc apparently wants to link with both, resulting in "multiple definitions of pthread\_..." linker errors.
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

The installed binary can now be executed.
It is located at `C:\msys64\mingw64\bin\rnote.exe` and depends on the environment provided by MSYS2.
It is not portable.

## Building the Installer

In order to build the installer, run the commands below.

```bash
meson compile rnote-gmo -C _mesonbuild
meson compile build-installer -C _mesonbuild
```

If successful, the generated installer will be located at `_mesonbuild/rnote-win-installer.exe`.
If you did not install MSYS2 into the default directory (`C:\msys64`),
then you will have to adjust the meson option called `msys-path` prior to building.

```bash
meson configure -Dmsys-path='C:\path\to\msys64' _mesonbuild
```

Likewise, you can adjust the output name of the installer using the `win-installer-name` option
(which defaults to `rnote_installer`).
