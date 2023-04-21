#!/usr/bin/env python3

import sys
import os
import shutil

source_root = sys.argv[1]
build_root = sys.argv[2]
inno_script = sys.argv[3]
msys_path = sys.argv[4]
app_id = sys.argv[5]
app_output = sys.argv[6]

print(f"""
### executing Inno-Setup installer build script with arguments: ###
    source_root: {source_root}
    build_root: {build_root}
    inno_script: {inno_script}
    msys_path: {msys_path}
    app_id: {app_id}
    app_output: {app_output}
""", file=sys.stderr)


# Collect DLLs
print("Collecting DLLs...", file=sys.stderr)
dlls_dir = os.path.join(build_root, "dlls/")

if os.path.exists(dlls_dir):
    shutil.rmtree(dlls_dir)

os.mkdir(dlls_dir)

# Don't use os.path.join here, because that uses the wrong separators which breaks wildcard expansion.
res = os.system(f"ldd {build_root}/{app_output} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}")
if res != 0:
    print("Collecting app DLL's failed, code: {res}")
    sys.exit(1)

res = os.system(f"ldd {msys_path}/mingw64/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}")
if res != 0:
    print("Collecting pixbuf-loaders DLL's failed, code: {res}")
    sys.exit(1)

# Collect necessary GSchema XML's and compile them into a `gschema.compiled`
print("Collecting and compiling GSchemas...", file=sys.stderr)
gschemas_dir = os.path.join(build_root, "gschemas/")

if os.path.exists(gschemas_dir):
    shutil.rmtree(gschemas_dir)

os.mkdir(gschemas_dir)

res = os.system(f"cp {msys_path}/mingw64/share/glib-2.0/schemas/org.gtk.* {gschemas_dir}")
if res != 0:
    print("Copying system schemas failed, code: {res}")
    sys.exit(1)

res = os.system(f"cp {build_root}/rnote-ui/data/{app_id}.gschema.xml {gschemas_dir}")
if res != 0:
    print("Copying app schema failed, code: {res}")
    sys.exit(1)

res = os.system(f"glib-compile-schemas {gschemas_dir}") # this generates `gschemas.compiled` in the same directory
if res != 0:
    print("Compiling schemas failed, code: {res}")
    sys.exit(1)

# Collect locale
print("Collecting locale...", file=sys.stderr)
locale_dir = os.path.join(build_root, "locale/")

if os.path.exists(locale_dir):
    shutil.rmtree(locale_dir)

os.mkdir(locale_dir)

# App locale
res = os.system(f"cp -r {build_root}/rnote-ui/po/. {locale_dir}")
if res != 0:
    print("Copying app locale failed, code: {res}")
    sys.exit(1)

# System locale
for file in os.listdir(os.path.join(build_root, "rnote-ui/po")):
    current_lang = os.fsdecode(file)
    current_locale_out_dir = os.path.join(locale_dir, f"{current_lang}/LC_MESSAGES")
    system_locale_dir = os.path.join(f"{msys_path}/mingw64/share/locale/{current_lang}/LC_MESSAGES")

    glib_locale = os.path.join(system_locale_dir, "glib20.mo")
    if os.path.exists(glib_locale):
        res = os.system(f"cp {glib_locale} {current_locale_out_dir}")
        if res != 0:
            print("Copying glib locale: {glib_locale} failed, code: {res}")
            sys.exit(1)

    gtk4_locale = os.path.join(system_locale_dir, "gtk40.mo")
    if os.path.exists(gtk4_locale):
        res = os.system(f"cp {gtk4_locale} {current_locale_out_dir}")
        if res != 0:
            print("Copying gtk4 locale: {gtk4_locale} failed, code: {res}")
            sys.exit(1)

    adw_locale = os.path.join(system_locale_dir, "libadwaita.mo")
    if os.path.exists(adw_locale):
        res = os.system(f"cp {adw_locale} {current_locale_out_dir}")
        if res != 0:
            print("Copying libadwaita locale: {adw_locale} failed, code: {res}")
            sys.exit(1)

    # TODO: do we need any other system locales?

# Build installer
print("Running ISCC...", file=sys.stderr)

res = os.system(f"{msys_path}/usr/bin/bash -c \"iscc '{inno_script}'\"")
if res != 0:
    print("Running iscc failed, code: {res}")
    sys.exit(1)

sys.exit(0)
