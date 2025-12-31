Template
```powershell
  # Downgrade gtk to avoid https://github.com/xournalpp/xournalpp/issues/6315
wget -q https://repo.msys2.org/mingw/${{inputs.msystem}}/mingw-w64-${{inputs.msys_package_env}}-gtk3-3.24.43-1-any.pkg.tar.zst
wget -q https://repo.msys2.org/mingw/${{inputs.msystem}}/mingw-w64-${{inputs.msys_package_env}}-gtk3-3.24.43-1-any.pkg.tar.zst.sig
pacman -U --noconfirm mingw-w64-${{inputs.msys_package_env}}-gtk3-3.24.43-1-any.pkg.tar.zst
```

cairo: 
    pkgver=1.18.4
    pkgrel=1
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-cairo-1.18.4-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-cairo-1.18.4-1-any.pkg.tar.zst.sig

gtk4: 
    pkgver=4.18.6
    pkgrel=3
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gtk4-4.18.6-3-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gtk4-4.18.6-3-any.pkg.tar.zst.sig

pango
    pkgver=1.56.4
    pkgrel=2
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-pango-1.56.4-2-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-pango-1.56.4-2-any.pkg.tar.zst.sig


appstream (there is also appstream-glib)
    pkgver=1.0.6
    pkgrel=2
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-appstream-1.0.6-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-appstream-1.0.6-1-any.pkg.tar.zst.sig

libadwaita
    pkgver=1.7.7
    pkgrel=1
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-libadwaita-1.7.7-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-libadwaita-1.7.7-1-any.pkg.tar.zst.sig

poppler
    pkgver=25.09.1
    pkgrel=1
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-poppler-25.09.1-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-poppler-25.09.1-1-any.pkg.tar.zst.sig

mingw-w64-dbus-glib ?
    pkgver=0.114
    pkgrel=3
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-dbus-glib-0.114-3-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-dbus-glib-0.114-3-any.pkg.tar.zst.sig

## TODO

glib2
    pkgver=2.86.0
    pkgrel=1

gdk-pixbuf2
    pkgver=2.42.12
    pkgrel=4

gnutls
    pkgver=3.8.10
    pkgrel=1

graphene
    pkgver=1.10.8
    pkgrel=2

harfbuzz
    pkgver=12.1.0
    pkgrel=1

librsvg
    pkgver=2.61.1
    pkgrel=1

ncurses
    _base_ver=6.5
    _date_rev=20250927
    pkgrel=1

libxmlb
    pkgver=0.3.24
    pkgrel=1

libtre
    pkgver=0.9.0
    pkgrel=1

libsystre
    pkgver=1.0.2
    pkgrel=1

libidn2
    _basever=2.3.8
    pkgrel=3

json-glib
    pkgver=1.10.8
    pkgrel=1
    
## Limited version test

From https://github.com/msys2/MINGW-packages/commit/3756eb5ceba81861751d26161a2ae6d980f715d3
    gettext: Don't export DllMain symbol
    So maybe I can limit this to gettext, cairo and gtk4 only
gettext
    pkgver=0.26
    pkgrel=1

- libtextstyle
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-libtextstyle-0.26-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-libtextstyle-0.26-1-any.pkg.tar.zst.sig

- runtime
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-runtime-0.26-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-runtime-0.26-1-any.pkg.tar.zst.sig

- tools
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-tools-0.26-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gettext-tools-0.26-1-any.pkg.tar.zst.sig

cairo: 
    pkgver=1.18.4
    pkgrel=1
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-cairo-1.18.4-1-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-cairo-1.18.4-1-any.pkg.tar.zst.sig

gtk4: 
    pkgver=4.18.6
    pkgrel=3
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gtk4-4.18.6-3-any.pkg.tar.zst
    https://repo.msys2.org/mingw/mingw64/mingw-w64-x86_64-gtk4-4.18.6-3-any.pkg.tar.zst.sig
