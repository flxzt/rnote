# Build instruction for Windows (experimental, WIP)

## Prerequisites
- install MSYS2 ( see: https://www.msys2.org/ )
- install Rust ( see: https://www.rust-lang.org/ )

mingw64's binary dir must be added to `Path`: Search for "Environment variables" and add C:\msys64\mingw64\bin` to the System "Path".

Then, in a msys2 terminal install the dependencies:
```bash
pacman -S git mingw-w64-x86_64-xz mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gcc mingw-w64-x86_64-clang mingw-w64-x86_64-toolchain mingw-w64-x86_64-autotools mingw-w64-x86_64-make mingw-w64-x86_64-cmake mingw-w64-x86_64-meson mingw-w64-x86_64-diffutils mingw-w64-x86_64-desktop-file-utils mingw-w64-x86_64-appstream-glib mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-poppler mingw-w64-x86_64-poppler-data
```

Add rust binary path to msys2's bash path by adding this line in `~/.bashrc`
```bash
export PATH=$PATH:/c/Users/<Username>/.cargo/bin
```

rusts toolchain needs to be changed:
```bash
rustup toolchain install stable-gnu
rustup default stable-gnu
```

then clone the repo somewhere and also init the submodules
```bash
git clone https://github.com/flxzt/rnote
git submodule update --init --recursive
```

## Building

Rnote is built with meson:

setup meson
```bash
meson setup --prefix=C:/msys64/mingw64 _mesonbuild
```

For reasons not yet understood, there are `libpthread.a` and `libpthread.dll.a` in `mingw64\lib\` and rustc apparently wants to link with both, resulting in "multiple definitions of pthread_..." linker errors. To solve this (in a very hacky way), rename `libpthread.dll.a` to `libpthread.dll.a.bak`.

Then the project can be compiled:

```bash
meson compile -C _mesonbuild
```

then install: 
```bash
meson install -C _mesonbuild
```

the installed binary can now be executed. It is located in `C:/msys64/mingw64/bin/rnote.exe`. It depends on the environment provided by mingw64, so it is not portable.
