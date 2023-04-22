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

def run_command(command, error_message):
    res = os.system(f"env MSYSTEM=MINGW64 {msys_path}/usr/bin/bash -lc '{command}'")
    if res != 0:
        print(f"{error_message}, code: {res}")
        print(f"command: {command}")
        sys.exit(1)


# Collect DLLs
print("Collecting DLLs...", file=sys.stderr)
dlls_dir = os.path.join(build_root, "dlls/")

if os.path.exists(dlls_dir):
    shutil.rmtree(dlls_dir)

os.mkdir(dlls_dir)

# Don't use os.path.join here, because that uses the wrong separators which breaks wildcard expansion.
run_command(
    f"ldd {build_root}/{app_output} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}",
    "Collecting app DLLs failed"
)

run_command(
    f"ldd {msys_path}/mingw64/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}",
    "Collecting pixbuf-loaders DLLs failed"
)

# Collect necessary GSchema XML's and compile them into a `gschema.compiled`
print("Collecting and compiling GSchemas...", file=sys.stderr)
gschemas_dir = os.path.join(build_root, "gschemas/")

if os.path.exists(gschemas_dir):
    shutil.rmtree(gschemas_dir)

os.mkdir(gschemas_dir)

run_command(
    f"cp {msys_path}/mingw64/share/glib-2.0/schemas/org.gtk.* {gschemas_dir}",
    "Copying system schemas failed"
)

run_command(
    f"cp {build_root}/rnote-ui/data/{app_id}.gschema.xml {gschemas_dir}",
    "Copying app schema failed"
)

run_command(
    f"glib-compile-schemas {gschemas_dir}", # this generates `gschemas.compiled` in the same directory
    "Compiling schemas failed"
)

# Collect locale
print("Collecting locale...", file=sys.stderr)
locale_dir = os.path.join(build_root, "locale/")

if os.path.exists(locale_dir):
    shutil.rmtree(locale_dir)

os.mkdir(locale_dir)

# App locale
run_command(
    f"cp -R {build_root}/rnote-ui/po/* {locale_dir}",
    "Copying app locale failed"
)

# System locale
for file in os.listdir(os.path.join(build_root, "rnote-ui/po")):
    current_lang = os.fsdecode(file)
    current_locale_out_dir = os.path.join(locale_dir, f"{current_lang}/LC_MESSAGES")
    system_locale_dir = os.path.join(f"{msys_path}/mingw64/share/locale/{current_lang}/LC_MESSAGES")

    glib_locale = os.path.join(system_locale_dir, "glib20.mo")
    if os.path.exists(glib_locale):
        run_command(
            f"cp {glib_locale} {current_locale_out_dir}",
            f"Copying glib locale: {glib_locale} failed"
        )

    gtk4_locale = os.path.join(system_locale_dir, "gtk40.mo")
    if os.path.exists(gtk4_locale):
        run_command(
            f"cp {gtk4_locale} {current_locale_out_dir}",
            f"Copying gtk4 locale: {gtk4_locale} failed"
        )

    adw_locale = os.path.join(system_locale_dir, "libadwaita.mo")
    if os.path.exists(adw_locale):
        run_command(
            f"cp {adw_locale} {current_locale_out_dir}",
            f"Copying libadwaita locale: {adw_locale} failed"
        )

    # TODO: do we need any other system locales?

# Build installer
print("Running ISCC...", file=sys.stderr)

run_command(
    f"iscc '{inno_script}'",
    "Running iscc failed"
)

sys.exit(0)
