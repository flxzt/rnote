#!/usr/bin/env python3

import sys
import os

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
dll_directory = os.path.join(build_root, "dlls/")

if not os.path.exists(dll_directory):
    print("Creating DLL directory...", file=sys.stderr)
    os.mkdir(dll_directory)

# Don't use os.path.join here, because that uses the wrong separators which breaks wildcard expansion.
print("Collecting DLLs...", file=sys.stderr)
os.system(f"ldd {build_root}/{app_output} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dll_directory}")
os.system(f"ldd {msys_path}/mingw64/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dll_directory}")

# Collect necessary GSchema XML's and compile them into a `gschema.compiled`
gschemas_directory = os.path.join(build_root, "gschemas/")

if not os.path.exists(gschemas_directory):
    print("Creating GSchemas directory...", file=sys.stderr)
    os.mkdir(gschemas_directory)

print("Collect and compile GSchemas", file=sys.stderr)
os.system(f"cp {msys_path}/mingw64/share/glib-2.0/schemas/org.gtk.* {gschemas_directory}")
os.system(f"cp {build_root}/rnote-ui/data/{app_id}.gschema.xml {gschemas_directory}")
os.system(f"glib-compile-schemas {gschemas_directory}") # this generates the `gschemas.compiled`

# Collect locale
locale_directory = os.path.join(build_root, "locale/")

if not os.path.exists(locale_directory):
    print("Creating locale directory...", file=sys.stderr)
    os.mkdir(locale_directory)

print("Collect locale", file=sys.stderr)
os.system(f"cp -R {build_root}/rnote-ui/po/* {locale_directory}")
# TODO collect gtk4 locale

print("Running ISCC...", file=sys.stderr)
os.system(f"{msys_path}/usr/bin/bash -c \"iscc '{inno_script}'\"")
