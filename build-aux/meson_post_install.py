#!/usr/bin/env python3

from os import environ, path
from subprocess import call

if not environ.get('DESTDIR', ''):
    PREFIX = environ.get('MESON_INSTALL_PREFIX', '/usr/local')
    DATA_DIR = path.join(PREFIX, 'share')

    if sys.platform.startswith('linux'):
        print('Updating icon cache...')
        call(['gtk-update-icon-cache', '-qtf', path.join(DATA_DIR, 'icons/hicolor')])
        print("Compiling new schemas...")
        call(["glib-compile-schemas", path.join(DATA_DIR, 'glib-2.0/schemas')])
        print("Updating desktop database...")
        call(["update-desktop-database", path.join(DATA_DIR, 'applications')])
        print("Updating MIME-type database...")
        call(["update-mime-database", path.join(DATA_DIR, 'mime')])
    elif sys.platform == "win32" or sys.platform == "cygwin":
        print('Updating icon cache...')
        call(['gtk-update-icon-cache-3.0.exe', '-qtf', path.join(DATA_DIR, 'icons/hicolor')])
        print("Compiling new schemas...")
        call(["glib-compile-schemas.exe", path.join(DATA_DIR, 'glib-2.0/schemas')])
        print("Updating desktop database...")
        call(["update-desktop-database.exe", path.join(DATA_DIR, 'applications')])
        print("Updating MIME-type database...")
        call(["update-mime-database.exe", path.join(DATA_DIR, 'mime')])
    else:
        print(f"[WARNING] \"meson_post_install.py\" does not work on {sys.platform}")
