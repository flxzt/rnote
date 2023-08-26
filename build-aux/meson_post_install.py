#!/usr/bin/env python3

import sys
import os
from subprocess import call

print("### executing post-install script ###", file=sys.stderr)

datadir = sys.argv[1]
bindir = sys.argv[2]
app_bin = sys.argv[3]

if not os.environ.get("DESTDIR", ""):
    if sys.platform.startswith("linux") or sys.platform.startswith("darwin"):
        print("Updating icon cache...", file=sys.stderr)
        call(["gtk4-update-icon-cache", "-qtf", os.path.join(datadir, "icons/hicolor")])
        print("Compiling new schemas...", file=sys.stderr)
        call(["glib-compile-schemas", os.path.join(datadir, "glib-2.0/schemas")])
        print("Updating desktop database...", file=sys.stderr)
        call(["update-desktop-database", os.path.join(datadir, "applications")])
        print("Updating MIME-type database...", file=sys.stderr)
        call(["update-mime-database", os.path.join(datadir, "mime")])
        print("Rebuilding font cache...", file=sys.stderr)
        call(["fc-cache", "-v", "-f"])
    elif sys.platform == "win32":
        print("Updating icon cache...", file=sys.stderr)
        call(["gtk-update-icon-cache.exe", "-qtf", os.path.join(datadir, "icons/hicolor")])
        print("Compiling new schemas...", file=sys.stderr)
        call(["glib-compile-schemas.exe", os.path.join(datadir, "glib-2.0/schemas")])
        print("Updating desktop database...", file=sys.stderr)
        call(["update-desktop-database.exe", os.path.join(datadir, "applications")])
        print("Updating MIME-type database...", file=sys.stderr)
        call(["update-mime-database.exe", os.path.join(datadir, "mime")])
        print("Rebuilding font cache...", file=sys.stderr)
        call(["fc-cache.exe", "-v", "-f"])
    else:
        print(f"[WARNING] \"meson_post_install.py\" is not configured to run on platform: {sys.platform}", file=sys.stderr)

print("### post-install script finished ###", file=sys.stderr)
