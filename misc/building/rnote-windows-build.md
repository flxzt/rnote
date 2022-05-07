# Build instruction for Windows (experimental, WIP)

### Prerequisites
- install msys2 from here: https://www.msys2.org/
- install Rust from here: https://www.rust-lang.org/

in a msys2 terminal, install git and the dependencies:
```
pacman -S git mingw-w64-x86_64-pkgconf mingw-w64-x86_64-gcc mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-poppler mingw-w64-x86_64-poppler-data
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

then clone the repo somwhere and also fetch the submodules
```
git clone https://github.com/flxzt/rnote
git submodule update --init --recursive
```

in the repo `build-aux/meson_post_install.py` must be overwritten with ( removing the desktop file check ):
```
#!/usr/bin/env python3

from os import environ, path
from subprocess import call

if not environ.get('DESTDIR', ''):
    PREFIX = environ.get('MESON_INSTALL_PREFIX', '/usr/local')
    DATA_DIR = path.join(PREFIX, 'share')
    print(DATA_DIR)
    print('Updating icon cache...')
    call(['gtk-update-icon-cache-3.0.exe', '-qtf', path.join(DATA_DIR, 'icons/hicolor')])
    print("Compiling new schemas...")
    call(["glib-compile-schemas.exe", path.join(DATA_DIR, 'glib-2.0/schemas')])
    print("Updating MIME-type database...")
    call(["update-mime-database.exe", path.join(DATA_DIR, 'mime')])
```

then Rnote can built with meson inside a **mingw64 shell**:
```
meson setup --prefix=C:/gnome _mesonbuild
meson compile -C _mesonbuild
```

then in `_mesonbuild/rnote.exe` rename to `rnote` .

then install: 
```
meson install -C _mesonbuild
```

and execute from the prefix binary path:
the binary should be installed in `C:/gnome/bin/rnote`. To execute rename it to `rnote.exe`