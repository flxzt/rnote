#!/usr/bin/env python3

import sys
import os
from subprocess import call

print("--- entering post-install script ---")

datadir = sys.argv[1]
bindir = sys.argv[2]
app_bin = sys.argv[3]

if not os.environ.get('DESTDIR', ''):
    if sys.platform.startswith('linux'):
        print('Updating icon cache...')
        call(['gtk-update-icon-cache', '-qtf', os.path.join(datadir, 'icons/hicolor')])
        print("Compiling new schemas...")
        call(["glib-compile-schemas", os.path.join(datadir, 'glib-2.0/schemas')])
        print("Updating desktop database...")
        call(["update-desktop-database", os.path.join(datadir, 'applications')])
        print("Updating MIME-type database...")
        call(["update-mime-database", os.path.join(datadir, 'mime')])
    elif sys.platform == "win32" or sys.platform == "cygwin":
        print('Updating icon cache...')
        call(['gtk-update-icon-cache.exe', '-qtf', os.path.join(datadir, 'icons/hicolor')])
        print("Compiling new schemas...")
        call(["glib-compile-schemas.exe", os.path.join(datadir, 'glib-2.0/schemas')])
        print("Updating desktop database...")
        call(["update-desktop-database.exe", os.path.join(datadir, 'applications')])
        print("Updating MIME-type database...")
        call(["update-mime-database.exe", os.path.join(datadir, 'mime')])
    else:
        print(f"[WARNING] \"meson_post_install.py\" is not configured to run on {sys.platform}")
