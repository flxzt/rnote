# Build instruction for Windows (experimental, WIP)

### Prerequisites
- install msys2 from here: https://www.msys2.org/
- install Rust from here: https://www.rust-lang.org/

in a msys2 terminal, install git and the dependencies:
```
pacman -S git mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gcc mingw-w64-x86_64-desktop-file-utils mingw-w64-x86_64-appstream-glib mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-poppler mingw-w64-x86_64-poppler-data
```

Add rust binary path to msys2's bash path: in `~/.bashrc`
```
export PATH=$PATH:/c/Users/<Username>/.cargo/bin
```

rusts toolchain needs to be changed:
```
rustup toolchain install stable-gnu
rustup default stable-gnu
```

then clone the repo somewhere and also init the submodules
```
git clone https://github.com/flxzt/rnote
git submodule update --init --recursive
```

Rnote can be built with meson inside a **mingw64 shell**:

setup meson
```
meson setup --prefix=C:/gnome _mesonbuild
```

compile
```
meson compile -C _mesonbuild
```

then install: 
```
meson install -C _mesonbuild
```

the installed binary can now be executed from binary path `C:/gnome/bin/rnote.exe`.
